use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingSystem {
    Windows,
    Linux,
    MacOs,
    Unknown(String),
}

impl fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperatingSystem::Windows => write!(f, "Windows"),
            OperatingSystem::Linux => write!(f, "Linux"),
            OperatingSystem::MacOs => write!(f, "macOS"),
            OperatingSystem::Unknown(s) => write!(f, "Unknown ({})", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Architecture {
    X86_64,
    Aarch64,
    Arm,
    X86,
    Unknown(String),
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::Aarch64 => write!(f, "aarch64"),
            Architecture::Arm => write!(f, "arm"),
            Architecture::X86 => write!(f, "x86"),
            Architecture::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionConfidence {
    Exact,
    High,
    Estimated,
    Unknown,
}

impl fmt::Display for DetectionConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionConfidence::Exact => write!(f, "exact"),
            DetectionConfidence::High => write!(f, "high"),
            DetectionConfidence::Estimated => write!(f, "estimated"),
            DetectionConfidence::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareSource {
    Os,
    CpuInfo,
    SystemApi,
    NvidiaSmi,
    Metal,
    Wmi,
    Dxgi,
    ProcFs,
    SysFs,
    Mock,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Unknown(String),
}

impl fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuVendor::Nvidia => write!(f, "NVIDIA"),
            GpuVendor::Amd => write!(f, "AMD"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Apple => write!(f, "Apple"),
            GpuVendor::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorCategory {
    DiscreteGpu,
    IntegratedGpu,
    UnifiedMemory,
    Npu,
    None,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CpuInfo {
    pub architecture: Architecture,
    pub logical_cores: u32,
    pub physical_cores: Option<u32>,
    pub vendor: Option<String>,
    pub model_name: Option<String>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub is_unified_memory: bool,
}

impl MemoryInfo {
    pub fn total_ram_mb(&self) -> u64 {
        self.total_bytes / (1024 * 1024)
    }

    pub fn available_ram_mb(&self) -> Option<u64> {
        self.available_bytes.map(|b| b / (1024 * 1024))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub name: String,
    pub vram_bytes: Option<u64>,
    pub available_vram_bytes: Option<u64>,
    pub driver_version: Option<String>,
    pub device_index: u32,
    pub accelerator_category: AcceleratorCategory,
}

impl GpuInfo {
    pub fn vram_mb(&self) -> Option<u64> {
        self.vram_bytes.map(|b| b / (1024 * 1024))
    }

    pub fn available_vram_mb(&self) -> Option<u64> {
        self.available_vram_bytes.map(|b| b / (1024 * 1024))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareProfile {
    pub os: OperatingSystem,
    pub os_detail: Option<String>,
    pub arch: Architecture,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpus: Vec<GpuInfo>,
    pub detected_at_timestamp: u64,
    pub confidence: DetectionConfidence,
    pub source: HardwareSource,
}

impl HardwareProfile {
    pub fn has_gpu(&self) -> bool {
        !self.gpus.is_empty()
    }

    pub fn total_vram_bytes(&self) -> u64 {
        self.gpus
            .iter()
            .filter_map(|g| g.vram_bytes)
            .fold(0u64, |acc, v| acc.saturating_add(v))
    }

    pub fn available_vram_bytes(&self) -> Option<u64> {
        let avail_list: Vec<u64> = self
            .gpus
            .iter()
            .filter_map(|g| g.available_vram_bytes)
            .collect();
        if avail_list.is_empty() {
            None
        } else {
            Some(
                avail_list
                    .iter()
                    .fold(0u64, |acc, v| acc.saturating_add(*v)),
            )
        }
    }

    pub fn primary_gpu(&self) -> Option<&GpuInfo> {
        self.gpus.first()
    }
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            os: OperatingSystem::Unknown("unknown".to_string()),
            os_detail: None,
            arch: Architecture::Unknown("unknown".to_string()),
            cpu: CpuInfo {
                architecture: Architecture::Unknown("unknown".to_string()),
                logical_cores: 4,
                physical_cores: None,
                vendor: None,
                model_name: None,
                features: vec![],
            },
            memory: MemoryInfo {
                total_bytes: 8 * 1024 * 1024 * 1024,
                available_bytes: Some(4 * 1024 * 1024 * 1024),
                used_bytes: Some(4 * 1024 * 1024 * 1024),
                is_unified_memory: false,
            },
            gpus: vec![],
            detected_at_timestamp: 0,
            confidence: DetectionConfidence::Estimated,
            source: HardwareSource::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GpuSnapshot {
    pub device_index: u32,
    pub name: String,
    pub vram_used_bytes: Option<u64>,
    pub vram_available_bytes: Option<u64>,
    pub utilization_percent: Option<f32>,
    pub temperature_celsius: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareSnapshot {
    pub timestamp: u64,
    pub total_ram_bytes: u64,
    pub available_ram_bytes: u64,
    pub ram_usage_percent: f32,
    pub cpu_usage_percent: f32,
    pub gpu_snapshots: Vec<GpuSnapshot>,
}
