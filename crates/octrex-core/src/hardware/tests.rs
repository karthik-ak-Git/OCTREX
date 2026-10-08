use crate::hardware::compatibility::{
    CompatibilityReason, CompatibilityStatus, ModelCompatibilityEngine,
};
use crate::hardware::fixtures::*;
use crate::hardware::profile::*;
use crate::models::capabilities::ModelCapability;
use crate::models::types::{
    HardwareRequirement, ModelAvailability, ModelDescriptor, TokenizerInfo,
};
use crate::providers::types::ExecutionMode;
use std::collections::HashMap;

fn create_test_model(
    id: &str,
    exec_mode: ExecutionMode,
    req: Option<HardwareRequirement>,
    metadata: HashMap<String, String>,
) -> ModelDescriptor {
    ModelDescriptor {
        id: id.to_string(),
        provider_id: "local_ollama".to_string(),
        model_identifier: id.to_string(),
        display_name: id.to_string(),
        execution_mode: exec_mode,
        capabilities: vec![ModelCapability::TextGeneration],
        context_window: Some(8192),
        max_output_tokens: Some(4096),
        tokenizer: TokenizerInfo::Unknown,
        hardware_requirements: req,
        availability: ModelAvailability::Available,
        metadata,
    }
}

#[tokio::test]
async fn test_1_cpu_detection_normalization() {
    let fix = fixture_a_8gb_nvidia();
    assert_eq!(fix.cpu.logical_cores, 8);
    assert_eq!(fix.cpu.physical_cores, Some(4));
    assert_eq!(fix.cpu.architecture, Architecture::X86_64);
}

#[tokio::test]
async fn test_2_ram_normalization() {
    let fix = fixture_a_8gb_nvidia();
    assert_eq!(fix.memory.total_bytes, 16 * 1024 * 1024 * 1024);
    assert_eq!(fix.memory.total_ram_mb(), 16384);
}

#[tokio::test]
async fn test_3_gpu_normalization() {
    let fix = fixture_a_8gb_nvidia();
    let gpu = fix.primary_gpu().unwrap();
    assert_eq!(gpu.vendor, GpuVendor::Nvidia);
    assert_eq!(gpu.vram_mb(), Some(8192));
}

#[tokio::test]
async fn test_4_no_gpu_machine() {
    let fix = fixture_c_cpu_only();
    assert!(!fix.has_gpu());
    assert_eq!(fix.gpus.len(), 0);
}

#[tokio::test]
async fn test_5_nvidia_gpu() {
    let fix = fixture_a_8gb_nvidia();
    assert_eq!(fix.gpus[0].vendor, GpuVendor::Nvidia);
}

#[tokio::test]
async fn test_6_amd_gpu() {
    let fix = fixture_e_16gb_amd();
    assert_eq!(fix.gpus[0].vendor, GpuVendor::Amd);
}

#[tokio::test]
async fn test_7_intel_gpu() {
    let mut fix = fixture_c_cpu_only();
    fix.gpus.push(GpuInfo {
        vendor: GpuVendor::Intel,
        name: "Intel Arc A770".to_string(),
        vram_bytes: Some(16 * 1024 * 1024 * 1024),
        available_vram_bytes: Some(14 * 1024 * 1024 * 1024),
        driver_version: None,
        device_index: 0,
        accelerator_category: AcceleratorCategory::DiscreteGpu,
    });
    assert_eq!(fix.gpus[0].vendor, GpuVendor::Intel);
}

#[tokio::test]
async fn test_8_unknown_gpu() {
    let mut fix = fixture_c_cpu_only();
    fix.gpus.push(GpuInfo {
        vendor: GpuVendor::Unknown("Custom Accelerator".to_string()),
        name: "Generic Accelerator".to_string(),
        vram_bytes: None,
        available_vram_bytes: None,
        driver_version: None,
        device_index: 0,
        accelerator_category: AcceleratorCategory::Unknown,
    });
    assert!(matches!(fix.gpus[0].vendor, GpuVendor::Unknown(_)));
}

#[tokio::test]
async fn test_9_multi_gpu() {
    let mut fix = fixture_a_8gb_nvidia();
    fix.gpus.push(GpuInfo {
        vendor: GpuVendor::Nvidia,
        name: "NVIDIA GeForce RTX 3070".to_string(),
        vram_bytes: Some(8 * 1024 * 1024 * 1024),
        available_vram_bytes: Some(8 * 1024 * 1024 * 1024),
        driver_version: Some("535.104".to_string()),
        device_index: 1,
        accelerator_category: AcceleratorCategory::DiscreteGpu,
    });
    assert_eq!(fix.gpus.len(), 2);
}

#[tokio::test]
async fn test_10_unavailable_vram() {
    let mut fix = fixture_a_8gb_nvidia();
    fix.gpus[0].vram_bytes = None;
    fix.gpus[0].available_vram_bytes = None;
    assert_eq!(fix.gpus[0].vram_mb(), None);
}

#[tokio::test]
async fn test_11_unknown_ram() {
    let mut fix = fixture_c_cpu_only();
    fix.memory.available_bytes = None;
    assert_eq!(fix.memory.available_ram_mb(), None);
}

#[tokio::test]
async fn test_12_unsupported_platform() {
    let fix = fixture_a_8gb_nvidia(); // Windows
    let mut meta = HashMap::new();
    meta.insert("supported_platform".to_string(), "macos".to_string());
    let model = create_test_model("llama-3-8b", ExecutionMode::Local, None, meta);

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Incompatible);
    assert!(res
        .reasons
        .iter()
        .any(|r| matches!(r, CompatibilityReason::UnsupportedPlatform { .. })));
}

#[tokio::test]
async fn test_13_low_memory_machine() {
    let mut fix = fixture_c_cpu_only();
    fix.memory.total_bytes = 4 * 1024 * 1024 * 1024; // 4 GB RAM

    let req = HardwareRequirement {
        minimum_ram: Some(8192),
        recommended_ram: Some(16384),
        minimum_vram: None,
        recommended_vram: None,
    };
    let model = create_test_model(
        "deepseek-r1-7b",
        ExecutionMode::Local,
        Some(req),
        HashMap::new(),
    );

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Incompatible);
}

#[tokio::test]
async fn test_14_cpu_only_machine() {
    let fix = fixture_c_cpu_only();
    let req = HardwareRequirement {
        minimum_ram: Some(4096),
        recommended_ram: Some(8192),
        minimum_vram: None,
        recommended_vram: None,
    };
    let model = create_test_model(
        "small-local",
        ExecutionMode::Local,
        Some(req),
        HashMap::new(),
    );

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::CompatibleWithWarnings);
    assert!(res.suitable_devices.contains(&"CPU".to_string()));
}

#[tokio::test]
async fn test_15_4gb_gpu() {
    let fix = fixture_b_4gb_nvidia(); // 4 GB GPU
    let req = HardwareRequirement {
        minimum_ram: Some(8192),
        recommended_ram: None,
        minimum_vram: Some(6144), // requires 6 GB VRAM
        recommended_vram: None,
    };
    let model = create_test_model("7b-model", ExecutionMode::Local, Some(req), HashMap::new());

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::CompatibleWithWarnings); // Offload to CPU with warning
}

#[tokio::test]
async fn test_16_8gb_gpu() {
    let fix = fixture_a_8gb_nvidia(); // 8 GB GPU
    let req = HardwareRequirement {
        minimum_ram: Some(8192),
        recommended_ram: None,
        minimum_vram: Some(6144), // requires 6 GB VRAM
        recommended_vram: None,
    };
    let model = create_test_model("7b-model", ExecutionMode::Local, Some(req), HashMap::new());

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Compatible);
}

#[tokio::test]
async fn test_17_16gb_ram() {
    let fix = fixture_a_8gb_nvidia(); // 16 GB RAM
    let req = HardwareRequirement {
        minimum_ram: Some(8192),
        recommended_ram: Some(16384),
        minimum_vram: None,
        recommended_vram: None,
    };
    let model = create_test_model("13b-model", ExecutionMode::Local, Some(req), HashMap::new());

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert!(res.status != CompatibilityStatus::Incompatible);
}

#[tokio::test]
async fn test_18_insufficient_vram() {
    let fix = fixture_b_4gb_nvidia(); // 4 GB VRAM
    let mut meta = HashMap::new();
    meta.insert("gpu_required".to_string(), "true".to_string());
    let req = HardwareRequirement {
        minimum_ram: Some(8192),
        recommended_ram: None,
        minimum_vram: Some(8192), // requires 8 GB VRAM
        recommended_vram: None,
    };
    let model = create_test_model("gpu-heavy-model", ExecutionMode::Local, Some(req), meta);

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Incompatible);
    assert!(res
        .reasons
        .iter()
        .any(|r| matches!(r, CompatibilityReason::InsufficientVram { .. })));
}

#[tokio::test]
async fn test_19_sufficient_vram() {
    let fix = fixture_d_24gb_nvidia_workstation(); // 24 GB VRAM
    let req = HardwareRequirement {
        minimum_ram: Some(16384),
        recommended_ram: None,
        minimum_vram: Some(16384),
        recommended_vram: None,
    };
    let model = create_test_model("32b-model", ExecutionMode::Local, Some(req), HashMap::new());

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Compatible);
}

#[tokio::test]
async fn test_20_insufficient_ram() {
    let fix = fixture_c_cpu_only(); // 8 GB RAM
    let req = HardwareRequirement {
        minimum_ram: Some(16384),
        recommended_ram: None,
        minimum_vram: None,
        recommended_vram: None,
    };
    let model = create_test_model(
        "large-model",
        ExecutionMode::Local,
        Some(req),
        HashMap::new(),
    );

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Incompatible);
    assert!(res
        .reasons
        .iter()
        .any(|r| matches!(r, CompatibilityReason::InsufficientRam { .. })));
}

#[tokio::test]
async fn test_21_sufficient_ram() {
    let fix = fixture_e_16gb_amd(); // 32 GB RAM
    let req = HardwareRequirement {
        minimum_ram: Some(16384),
        recommended_ram: None,
        minimum_vram: None,
        recommended_vram: None,
    };
    let model = create_test_model(
        "medium-model",
        ExecutionMode::Local,
        Some(req),
        HashMap::new(),
    );

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Compatible);
}

#[tokio::test]
async fn test_22_gpu_required_model_with_no_gpu() {
    let fix = fixture_c_cpu_only(); // No GPU
    let mut meta = HashMap::new();
    meta.insert("gpu_required".to_string(), "true".to_string());
    let model = create_test_model("cuda-only-model", ExecutionMode::Local, None, meta);

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Incompatible);
    assert!(res
        .reasons
        .iter()
        .any(|r| matches!(r, CompatibilityReason::GpuRequiredButMissing)));
}

#[tokio::test]
async fn test_23_vendor_mismatch() {
    let fix = fixture_e_16gb_amd(); // AMD GPU
    let mut meta = HashMap::new();
    meta.insert("gpu_vendor".to_string(), "nvidia".to_string());
    let model = create_test_model("tensorrt-model", ExecutionMode::Local, None, meta);

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Incompatible);
    assert!(res
        .reasons
        .iter()
        .any(|r| matches!(r, CompatibilityReason::GpuVendorMismatch { .. })));
}

#[tokio::test]
async fn test_24_architecture_mismatch() {
    let fix = fixture_a_8gb_nvidia(); // x86_64
    let mut meta = HashMap::new();
    meta.insert("architecture".to_string(), "aarch64".to_string());
    let model = create_test_model("arm-binary-model", ExecutionMode::Local, None, meta);

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Incompatible);
}

#[tokio::test]
async fn test_25_platform_mismatch() {
    let fix = fixture_a_8gb_nvidia(); // Windows
    let mut meta = HashMap::new();
    meta.insert("supported_platform".to_string(), "linux".to_string());
    let model = create_test_model("linux-only-model", ExecutionMode::Local, None, meta);

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Incompatible);
}

#[tokio::test]
async fn test_26_unknown_model_requirements() {
    let fix = fixture_a_8gb_nvidia();
    let model = create_test_model("mystery-model", ExecutionMode::Local, None, HashMap::new());

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::CompatibleWithWarnings);
    assert!(res
        .reasons
        .iter()
        .any(|r| matches!(r, CompatibilityReason::UnknownRequirements)));
}

#[tokio::test]
async fn test_27_recommended_vs_minimum_requirements() {
    let fix = fixture_a_8gb_nvidia(); // 16 GB RAM
    let req = HardwareRequirement {
        minimum_ram: Some(8192),
        recommended_ram: Some(32768), // recommended 32 GB RAM
        minimum_vram: None,
        recommended_vram: None,
    };
    let model = create_test_model(
        "greedy-model",
        ExecutionMode::Local,
        Some(req),
        HashMap::new(),
    );

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::CompatibleWithWarnings);
    assert!(res
        .warnings
        .iter()
        .any(|w| w.contains("below recommended RAM")));
}

#[tokio::test]
async fn test_28_compatible_with_warning_state() {
    let fix = fixture_c_cpu_only();
    let model = create_test_model("cpu-model", ExecutionMode::Local, None, HashMap::new());

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::CompatibleWithWarnings);
}

#[tokio::test]
async fn test_29_incompatible_state() {
    let fix = fixture_c_cpu_only(); // 8 GB RAM
    let req = HardwareRequirement {
        minimum_ram: Some(65536), // requires 64 GB RAM
        recommended_ram: None,
        minimum_vram: None,
        recommended_vram: None,
    };
    let model = create_test_model(
        "giant-model",
        ExecutionMode::Local,
        Some(req),
        HashMap::new(),
    );

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert_eq!(res.status, CompatibilityStatus::Incompatible);
}

#[tokio::test]
async fn test_30_compatibility_reason_generation() {
    let fix = fixture_c_cpu_only();
    let req = HardwareRequirement {
        minimum_ram: Some(32768),
        recommended_ram: None,
        minimum_vram: None,
        recommended_vram: None,
    };
    let model = create_test_model("big-model", ExecutionMode::Local, Some(req), HashMap::new());

    let res = ModelCompatibilityEngine::evaluate(&fix, &model);
    assert!(!res.reasons.is_empty());
    assert!(matches!(
        res.reasons[0],
        CompatibilityReason::InsufficientRam { .. }
    ));
}
