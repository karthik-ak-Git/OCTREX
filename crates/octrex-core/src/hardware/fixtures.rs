use crate::hardware::profile::{
    AcceleratorCategory, Architecture, CpuInfo, DetectionConfidence, GpuInfo, GpuVendor,
    HardwareProfile, HardwareSource, MemoryInfo, OperatingSystem,
};

pub fn fixture_a_8gb_nvidia() -> HardwareProfile {
    HardwareProfile {
        os: OperatingSystem::Windows,
        os_detail: Some("Windows 11 Pro".to_string()),
        arch: Architecture::X86_64,
        cpu: CpuInfo {
            architecture: Architecture::X86_64,
            logical_cores: 8,
            physical_cores: Some(4),
            vendor: Some("GenuineIntel".to_string()),
            model_name: Some("Intel Core i7-10700K".to_string()),
            features: vec!["AVX2".to_string()],
        },
        memory: MemoryInfo {
            total_bytes: 16 * 1024 * 1024 * 1024,
            available_bytes: Some(12 * 1024 * 1024 * 1024),
            used_bytes: Some(4 * 1024 * 1024 * 1024),
            is_unified_memory: false,
        },
        gpus: vec![GpuInfo {
            vendor: GpuVendor::Nvidia,
            name: "NVIDIA GeForce RTX 3070".to_string(),
            vram_bytes: Some(8 * 1024 * 1024 * 1024),
            available_vram_bytes: Some(7 * 1024 * 1024 * 1024),
            driver_version: Some("535.104".to_string()),
            device_index: 0,
            accelerator_category: AcceleratorCategory::DiscreteGpu,
        }],
        detected_at_timestamp: 1700000000,
        confidence: DetectionConfidence::Exact,
        source: HardwareSource::NvidiaSmi,
    }
}

pub fn fixture_b_4gb_nvidia() -> HardwareProfile {
    HardwareProfile {
        os: OperatingSystem::Windows,
        os_detail: Some("Windows 10 Home".to_string()),
        arch: Architecture::X86_64,
        cpu: CpuInfo {
            architecture: Architecture::X86_64,
            logical_cores: 8,
            physical_cores: Some(4),
            vendor: Some("GenuineIntel".to_string()),
            model_name: Some("Intel Core i5-9400F".to_string()),
            features: vec!["AVX2".to_string()],
        },
        memory: MemoryInfo {
            total_bytes: 16 * 1024 * 1024 * 1024,
            available_bytes: Some(10 * 1024 * 1024 * 1024),
            used_bytes: Some(6 * 1024 * 1024 * 1024),
            is_unified_memory: false,
        },
        gpus: vec![GpuInfo {
            vendor: GpuVendor::Nvidia,
            name: "NVIDIA GeForce GTX 1650".to_string(),
            vram_bytes: Some(4 * 1024 * 1024 * 1024),
            available_vram_bytes: Some(3 * 1024 * 1024 * 1024),
            driver_version: Some("520.56".to_string()),
            device_index: 0,
            accelerator_category: AcceleratorCategory::DiscreteGpu,
        }],
        detected_at_timestamp: 1700000000,
        confidence: DetectionConfidence::Exact,
        source: HardwareSource::NvidiaSmi,
    }
}

pub fn fixture_c_cpu_only() -> HardwareProfile {
    HardwareProfile {
        os: OperatingSystem::Linux,
        os_detail: Some("Ubuntu 22.04 LTS".to_string()),
        arch: Architecture::X86_64,
        cpu: CpuInfo {
            architecture: Architecture::X86_64,
            logical_cores: 8,
            physical_cores: Some(4),
            vendor: Some("AuthenticAMD".to_string()),
            model_name: Some("AMD Ryzen 5 3600".to_string()),
            features: vec!["AVX2".to_string()],
        },
        memory: MemoryInfo {
            total_bytes: 8 * 1024 * 1024 * 1024,
            available_bytes: Some(6 * 1024 * 1024 * 1024),
            used_bytes: Some(2 * 1024 * 1024 * 1024),
            is_unified_memory: false,
        },
        gpus: vec![],
        detected_at_timestamp: 1700000000,
        confidence: DetectionConfidence::High,
        source: HardwareSource::SystemApi,
    }
}

pub fn fixture_d_24gb_nvidia_workstation() -> HardwareProfile {
    HardwareProfile {
        os: OperatingSystem::Linux,
        os_detail: Some("Ubuntu 24.04 LTS".to_string()),
        arch: Architecture::X86_64,
        cpu: CpuInfo {
            architecture: Architecture::X86_64,
            logical_cores: 16,
            physical_cores: Some(8),
            vendor: Some("AuthenticAMD".to_string()),
            model_name: Some("AMD Ryzen 7 7800X3D".to_string()),
            features: vec!["AVX2".to_string(), "AVX512".to_string()],
        },
        memory: MemoryInfo {
            total_bytes: 64 * 1024 * 1024 * 1024,
            available_bytes: Some(50 * 1024 * 1024 * 1024),
            used_bytes: Some(14 * 1024 * 1024 * 1024),
            is_unified_memory: false,
        },
        gpus: vec![GpuInfo {
            vendor: GpuVendor::Nvidia,
            name: "NVIDIA GeForce RTX 4090".to_string(),
            vram_bytes: Some(24 * 1024 * 1024 * 1024),
            available_vram_bytes: Some(22 * 1024 * 1024 * 1024),
            driver_version: Some("550.54".to_string()),
            device_index: 0,
            accelerator_category: AcceleratorCategory::DiscreteGpu,
        }],
        detected_at_timestamp: 1700000000,
        confidence: DetectionConfidence::Exact,
        source: HardwareSource::NvidiaSmi,
    }
}

pub fn fixture_e_16gb_amd() -> HardwareProfile {
    HardwareProfile {
        os: OperatingSystem::Linux,
        os_detail: Some("Fedora Workstation 39".to_string()),
        arch: Architecture::X86_64,
        cpu: CpuInfo {
            architecture: Architecture::X86_64,
            logical_cores: 8,
            physical_cores: Some(4),
            vendor: Some("AuthenticAMD".to_string()),
            model_name: Some("AMD Ryzen 7 5700X".to_string()),
            features: vec!["AVX2".to_string()],
        },
        memory: MemoryInfo {
            total_bytes: 32 * 1024 * 1024 * 1024,
            available_bytes: Some(24 * 1024 * 1024 * 1024),
            used_bytes: Some(8 * 1024 * 1024 * 1024),
            is_unified_memory: false,
        },
        gpus: vec![GpuInfo {
            vendor: GpuVendor::Amd,
            name: "AMD Radeon RX 7800 XT".to_string(),
            vram_bytes: Some(16 * 1024 * 1024 * 1024),
            available_vram_bytes: Some(15 * 1024 * 1024 * 1024),
            driver_version: Some("23.12.1".to_string()),
            device_index: 0,
            accelerator_category: AcceleratorCategory::DiscreteGpu,
        }],
        detected_at_timestamp: 1700000000,
        confidence: DetectionConfidence::High,
        source: HardwareSource::SystemApi,
    }
}
