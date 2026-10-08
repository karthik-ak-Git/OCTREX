use crate::hardware::profile::{DetectionConfidence, HardwareProfile};
use crate::models::types::ModelDescriptor;
use crate::providers::types::ExecutionMode;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    Compatible,
    CompatibleWithWarnings,
    Incompatible,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum CompatibilityReason {
    InsufficientVram { required_mb: u64, available_mb: u64 },
    InsufficientRam { required_mb: u64, available_mb: u64 },
    GpuRequiredButMissing,
    GpuVendorMismatch { required: String, actual: String },
    UnsupportedArchitecture { required: String, actual: String },
    UnsupportedPlatform { required: String, actual: String },
    UnknownRequirements,
    CpuOnlyConstrained { note: String },
    SufficientResources,
    RecommendedResourcesExceeded { details: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompatibilityResult {
    pub model_id: String,
    pub status: CompatibilityStatus,
    pub reasons: Vec<CompatibilityReason>,
    pub warnings: Vec<String>,
    pub estimated_constraints: Vec<String>,
    pub suitable_devices: Vec<String>,
    pub confidence: DetectionConfidence,
    pub evaluated_at_timestamp: u64,
}

pub struct ModelCompatibilityEngine;

impl ModelCompatibilityEngine {
    pub fn evaluate(profile: &HardwareProfile, model: &ModelDescriptor) -> CompatibilityResult {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut reasons = Vec::new();
        let mut warnings = Vec::new();
        let mut estimated_constraints = Vec::new();
        let mut suitable_devices = Vec::new();
        let mut is_incompatible = false;
        let mut is_warning = false;

        let reqs = model.hardware_requirements.clone();

        // Metadata checks for extra constraints if present
        let gpu_required = model
            .metadata
            .get("gpu_required")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let required_vendor = model.metadata.get("gpu_vendor").cloned();
        let required_arch = model.metadata.get("architecture").cloned();
        let required_platform = model.metadata.get("supported_platform").cloned();

        // 1. Cloud models: always compatible from hardware perspective
        if model.execution_mode == ExecutionMode::Cloud {
            return CompatibilityResult {
                model_id: model.id.clone(),
                status: CompatibilityStatus::Compatible,
                reasons: vec![CompatibilityReason::SufficientResources],
                warnings: vec![],
                estimated_constraints: vec![],
                suitable_devices: vec!["Cloud Infrastructure".to_string()],
                confidence: DetectionConfidence::Exact,
                evaluated_at_timestamp: timestamp,
            };
        }

        // 2. Check Platform compatibility
        if let Some(req_plat) = &required_platform {
            let current_os = profile.os.to_string().to_lowercase();
            if !current_os.contains(&req_plat.to_lowercase()) {
                is_incompatible = true;
                reasons.push(CompatibilityReason::UnsupportedPlatform {
                    required: req_plat.clone(),
                    actual: profile.os.to_string(),
                });
            }
        }

        // 3. Check Architecture compatibility
        if let Some(req_a) = &required_arch {
            let current_arch = profile.arch.to_string().to_lowercase();
            if !current_arch.contains(&req_a.to_lowercase()) {
                is_incompatible = true;
                reasons.push(CompatibilityReason::UnsupportedArchitecture {
                    required: req_a.clone(),
                    actual: profile.arch.to_string(),
                });
            }
        }

        // 4. Check GPU requirement
        if gpu_required && !profile.has_gpu() {
            is_incompatible = true;
            reasons.push(CompatibilityReason::GpuRequiredButMissing);
        }

        // 5. Check GPU Vendor mismatch if required
        if let Some(req_vendor_str) = &required_vendor {
            if profile.has_gpu() {
                let matching = profile.gpus.iter().any(|g| {
                    g.vendor
                        .to_string()
                        .to_lowercase()
                        .contains(&req_vendor_str.to_lowercase())
                });
                if !matching {
                    is_incompatible = true;
                    let actual_vendors: Vec<String> =
                        profile.gpus.iter().map(|g| g.vendor.to_string()).collect();
                    reasons.push(CompatibilityReason::GpuVendorMismatch {
                        required: req_vendor_str.clone(),
                        actual: actual_vendors.join(", "),
                    });
                }
            }
        }

        // 6. RAM requirement evaluation
        if let Some(req) = &reqs {
            if let Some(min_ram_mb) = req.minimum_ram {
                let available_ram_mb = profile.memory.total_ram_mb();
                if available_ram_mb < min_ram_mb {
                    is_incompatible = true;
                    reasons.push(CompatibilityReason::InsufficientRam {
                        required_mb: min_ram_mb,
                        available_mb: available_ram_mb,
                    });
                } else if let Some(rec_ram_mb) = req.recommended_ram {
                    if available_ram_mb < rec_ram_mb {
                        is_warning = true;
                        warnings.push(format!(
                            "System RAM ({} MB) is below recommended RAM ({} MB)",
                            available_ram_mb, rec_ram_mb
                        ));
                        estimated_constraints
                            .push("Higher memory usage during peak context length".to_string());
                    }
                }
            }

            // 7. VRAM requirement evaluation
            if let Some(min_vram_mb) = req.minimum_vram {
                if !profile.has_gpu() {
                    if gpu_required {
                        is_incompatible = true;
                    } else {
                        is_warning = true;
                        warnings.push(
                            "Model specifies VRAM requirement, but no GPU was detected; falling back to CPU execution"
                                .to_string(),
                        );
                    }
                } else {
                    // Evaluate suitable GPUs
                    let mut found_suitable = false;
                    for gpu in &profile.gpus {
                        let gpu_vram_mb = gpu.vram_mb().or_else(|| {
                            if profile.memory.is_unified_memory {
                                Some(profile.memory.total_ram_mb())
                            } else {
                                None
                            }
                        });

                        if let Some(vram_mb) = gpu_vram_mb {
                            if vram_mb >= min_vram_mb {
                                found_suitable = true;
                                suitable_devices.push(format!(
                                    "GPU {}: {} ({} MB VRAM)",
                                    gpu.device_index, gpu.name, vram_mb
                                ));
                            }
                        }
                    }

                    if !found_suitable {
                        let max_vram_mb = profile
                            .gpus
                            .iter()
                            .filter_map(|g| g.vram_mb())
                            .max()
                            .unwrap_or(0);

                        if gpu_required {
                            is_incompatible = true;
                            reasons.push(CompatibilityReason::InsufficientVram {
                                required_mb: min_vram_mb,
                                available_mb: max_vram_mb,
                            });
                        } else {
                            is_warning = true;
                            reasons.push(CompatibilityReason::InsufficientVram {
                                required_mb: min_vram_mb,
                                available_mb: max_vram_mb,
                            });
                            warnings.push(format!(
                                "Available VRAM ({} MB) is below required VRAM ({} MB); execution will offload to CPU",
                                max_vram_mb, min_vram_mb
                            ));
                            estimated_constraints.push(
                                "Hybrid CPU/GPU offloading will reduce throughput".to_string(),
                            );
                        }
                    }
                }
            }
        } else {
            // No explicit hardware_requirements specified
            is_warning = true;
            reasons.push(CompatibilityReason::UnknownRequirements);
            warnings.push(
                "Model requirements are unspecified; hardware capability estimated".to_string(),
            );
        }

        // 8. CPU-only constraint warning
        if !profile.has_gpu() && !is_incompatible {
            is_warning = true;
            suitable_devices.push("CPU".to_string());
            reasons.push(CompatibilityReason::CpuOnlyConstrained {
                note: "No GPU detected; local execution relies solely on CPU".to_string(),
            });
            warnings.push(
                "CPU-only execution expected to be constrained; inference latency will be higher"
                    .to_string(),
            );
            estimated_constraints
                .push("Reduced tokens-per-second generation rate on CPU".to_string());
        } else if profile.has_gpu() && suitable_devices.is_empty() {
            // Default suitable devices to all available GPUs if compatible
            for gpu in &profile.gpus {
                suitable_devices.push(format!("GPU {}: {}", gpu.device_index, gpu.name));
            }
        }

        if reasons.is_empty() {
            reasons.push(CompatibilityReason::SufficientResources);
        }

        let status = if is_incompatible {
            CompatibilityStatus::Incompatible
        } else if is_warning {
            CompatibilityStatus::CompatibleWithWarnings
        } else {
            CompatibilityStatus::Compatible
        };

        CompatibilityResult {
            model_id: model.id.clone(),
            status,
            reasons,
            warnings,
            estimated_constraints,
            suitable_devices,
            confidence: profile.confidence,
            evaluated_at_timestamp: timestamp,
        }
    }
}
