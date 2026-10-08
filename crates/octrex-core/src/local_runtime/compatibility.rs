use crate::context::budget::{BudgetPolicy, TokenBudget};
use crate::hardware::{HardwareProfile, ModelCompatibilityEngine};
use crate::local_runtime::types::{LocalCompatibilityReport, ResourceEstimate};
use crate::models::ModelDescriptor;

/// Conservative bytes-per-parameter table (MB per billion params).
///
/// Documented assumptions:
/// - fp16/bf16 ~ 2 bytes/param, fp32 ~ 4, int8 ~ 1, int4 ~ 0.55 (with
///   quantization overhead), unknown defaults to 2 (fp16-equivalent).
/// - Runtime overhead is a flat reservation, not a precise measurement.
/// - KV cache scales with context length; we estimate linearly against a
///   4k-token baseline because exact attention math is model-specific.
/// - Every uncertain value is flagged `is_estimate = true` and surfaced with
///   warnings. These numbers gate safety decisions conservatively: when in
///   doubt the estimate is rounded UP so hardware is never over-promised.
pub const RUNTIME_OVERHEAD_MB: u64 = 512;
pub const KV_CACHE_BASELINE_CTX: u32 = 4096;
pub const KV_CACHE_BASELINE_MB_PER_B: f64 = 180.0;

pub fn bytes_per_param_mb(quantization: Option<&str>) -> f64 {
    match quantization
        .unwrap_or("unknown")
        .to_lowercase()
        .replace(['-', '_', ' '], "")
        .as_str()
    {
        "fp32" | "f32" => 4000.0,
        "fp16" | "f16" | "bf16" => 2000.0,
        "int8" | "q8" | "q80" | "q8_0" => 1000.0,
        "int4" | "q4" | "q40" | "q4_0" | "q4k" | "q4km" | "nf4" => 550.0,
        "q5" | "q50" | "q5k" | "q5km" => 700.0,
        "q6" | "q60" | "q6k" => 850.0,
        "q3" | "q30" => 450.0,
        "q2" => 350.0,
        _ => 2000.0,
    }
}

/// Estimate RAM/VRAM needs for a model record.
///
/// Returns `None` for both estimates when parameter count is unknown — the
/// caller must then treat requirements as UNKNOWN, never as compatible.
#[allow(clippy::too_many_arguments)]
pub fn estimate_resources(
    model_id: &str,
    parameter_count_billions: Option<f64>,
    quantization: Option<&str>,
    context_window: Option<u32>,
    known_ram_mb: Option<u64>,
    known_vram_mb: Option<u64>,
) -> ResourceEstimate {
    let mut assumptions = vec![
        "Estimates are conservative upper bounds, not measurements.".to_string(),
        format!(
            "Runtime overhead reservation: {} MB (llama.cpp-class server + HTTP layer).",
            RUNTIME_OVERHEAD_MB
        ),
    ];
    let mut warnings = Vec::new();

    let (est_ram, est_vram, kv_mb, confidence) = match parameter_count_billions {
        Some(params_b) if params_b > 0.0 => {
            let per_b = bytes_per_param_mb(quantization);
            assumptions.push(format!(
                "Weight footprint assumes {:.0} MB per billion params (quantization '{}').",
                per_b,
                quantization.unwrap_or("unknown")
            ));
            let weights_mb = (params_b * per_b).ceil() as u64;
            let ctx = context_window.unwrap_or(KV_CACHE_BASELINE_CTX);
            let ctx_scale = ctx as f64 / KV_CACHE_BASELINE_CTX as f64;
            let kv = (params_b * KV_CACHE_BASELINE_MB_PER_B * ctx_scale).ceil() as u64;
            assumptions.push(format!(
                "KV cache scales linearly with context ({} tokens vs {} baseline).",
                ctx, KV_CACHE_BASELINE_CTX
            ));
            if quantization.is_none() {
                warnings.push(
                    "Quantization unknown: assumed fp16-equivalent footprint (upper bound)."
                        .to_string(),
                );
            }
            if context_window.is_none() {
                warnings.push(
                    "Context window unknown: KV cache estimated at 4k-token baseline.".to_string(),
                );
            }
            let ram = weights_mb + kv + RUNTIME_OVERHEAD_MB;
            // VRAM estimate assumes full GPU offload where a GPU exists;
            // CPU-only hosts use RAM instead (handled by the caller).
            let vram = weights_mb + kv / 2 + RUNTIME_OVERHEAD_MB / 2;
            (Some(ram), Some(vram), Some(kv), "estimated")
        }
        _ => {
            warnings.push(
                "Parameter count unknown: resource requirements cannot be estimated.".to_string(),
            );
            (None, None, None, "unknown")
        }
    };

    // Known values always win over estimates and are marked exact.
    let (final_ram, final_vram, is_estimate) = match (known_ram_mb, known_vram_mb) {
        (Some(r), Some(v)) => (Some(r), Some(v), false),
        (Some(r), None) => (Some(r), est_vram, true),
        (None, Some(v)) => (est_ram, Some(v), true),
        (None, None) => (est_ram, est_vram, true),
    };
    if known_ram_mb.is_some() || known_vram_mb.is_some() {
        assumptions
            .push("Runtime-reported requirements take precedence over estimates.".to_string());
    }

    ResourceEstimate {
        model_id: model_id.to_string(),
        estimated_ram_mb: final_ram,
        estimated_vram_mb: final_vram,
        estimated_kv_cache_mb: kv_mb,
        runtime_overhead_mb: RUNTIME_OVERHEAD_MB,
        is_estimate,
        confidence: confidence.to_string(),
        assumptions,
        warnings,
    }
}

/// Full local compatibility evaluation.
///
/// Reuses the Phase 5 `ModelCompatibilityEngine` for hardware and the Phase 10
/// `TokenBudget` invariant for context:
///
/// ```text
/// input + reserved_output + safety_margin + fixed_overhead <= context_window
/// ```
///
/// A model is routable only when hardware is `Compatible` (warnings allowed)
/// AND the context budget fits AND lifecycle/health permit it (checked by the
/// caller via `routable`).
pub fn evaluate_local_compatibility(
    profile: &HardwareProfile,
    descriptor: &ModelDescriptor,
    runtime_id: &str,
    required_input_tokens: usize,
    required_output_tokens: Option<usize>,
    parameter_count_billions: Option<f64>,
    quantization: Option<&str>,
) -> LocalCompatibilityReport {
    let hardware = ModelCompatibilityEngine::evaluate(profile, descriptor);
    let mut reasons: Vec<String> = Vec::new();
    for r in &hardware.reasons {
        reasons.push(format!("{:?}", r));
    }
    for w in &hardware.warnings {
        reasons.push(format!("warning: {}", w));
    }

    let estimate = estimate_resources(
        &descriptor.id,
        parameter_count_billions,
        quantization,
        descriptor.context_window,
        descriptor
            .hardware_requirements
            .as_ref()
            .and_then(|h| h.minimum_ram),
        descriptor
            .hardware_requirements
            .as_ref()
            .and_then(|h| h.minimum_vram),
    );

    // Context compatibility via the existing TokenBudget engine.
    let (context_ok, context_detail) = match descriptor.context_window {
        Some(window) if window > 0 => {
            let policy = BudgetPolicy::default();
            match TokenBudget::compute(
                &descriptor.id,
                window as usize,
                required_output_tokens,
                &policy,
            ) {
                Ok(budget) => {
                    if budget.would_overflow(required_input_tokens) {
                        (
                            false,
                            format!(
                                "Context budget exceeded: need {} input tokens, usable budget is {} (window {})",
                                required_input_tokens,
                                budget.usable_input_budget,
                                window
                            ),
                        )
                    } else {
                        (
                            true,
                            format!(
                                "Context fits: {} input tokens within {} usable budget (window {})",
                                required_input_tokens, budget.usable_input_budget, window
                            ),
                        )
                    }
                }
                Err(e) => (false, format!("Context budget error: {}", e)),
            }
        }
        _ => (
            false,
            "Context window unknown: cannot prove the request fits; treated as incompatible."
                .to_string(),
        ),
    };
    if !context_ok {
        reasons.push(format!("context: {}", context_detail));
    }

    let hardware_ok = matches!(
        hardware.status,
        crate::hardware::CompatibilityStatus::Compatible
            | crate::hardware::CompatibilityStatus::CompatibleWithWarnings
    );
    // Unknown hardware status never auto-promotes to routable.
    let routable = hardware_ok && context_ok;

    LocalCompatibilityReport {
        model_id: descriptor.id.clone(),
        runtime_id: runtime_id.to_string(),
        hardware,
        context_ok,
        context_detail,
        resource_estimate: estimate,
        routable,
        reasons,
    }
}
