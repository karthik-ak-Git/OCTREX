use crate::hardware::profile::{Architecture, CpuInfo};
use sysinfo::System;

pub fn detect_cpu_info(sys: &System) -> CpuInfo {
    let arch = match std::env::consts::ARCH {
        "x86_64" => Architecture::X86_64,
        "aarch64" => Architecture::Aarch64,
        "arm" => Architecture::Arm,
        "x86" => Architecture::X86,
        other => Architecture::Unknown(other.to_string()),
    };

    let logical_cores = sys.cpus().len() as u32;
    let physical_cores = sys.physical_core_count().map(|c| c as u32);

    let (vendor, model_name) = if let Some(cpu) = sys.cpus().first() {
        let vendor = if !cpu.vendor_id().is_empty() {
            Some(cpu.vendor_id().to_string())
        } else {
            None
        };
        let model = if !cpu.brand().is_empty() {
            Some(cpu.brand().to_string())
        } else {
            None
        };
        (vendor, model)
    } else {
        (None, None)
    };

    let mut features = Vec::new();
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            features.push("AVX2".to_string());
        }
        if is_x86_feature_detected!("avx") {
            features.push("AVX".to_string());
        }
        if is_x86_feature_detected!("fma") {
            features.push("FMA".to_string());
        }
    }

    CpuInfo {
        architecture: arch,
        logical_cores: if logical_cores > 0 { logical_cores } else { 1 },
        physical_cores,
        vendor,
        model_name,
        features,
    }
}
