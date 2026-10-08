pub mod compatibility;
pub mod cpu;
pub mod detector;
pub mod errors;
pub mod fixtures;
pub mod gpu;
pub mod memory;
pub mod platform;
pub mod profile;
pub mod profiler;

#[cfg(test)]
pub mod tests;

pub use compatibility::{
    CompatibilityReason, CompatibilityResult, CompatibilityStatus, ModelCompatibilityEngine,
};
pub use cpu::detect_cpu_info;
pub use detector::{HardwareProbe, HardwareService, MockHardwareProbe, RealHardwareProbe};
pub use errors::HardwareError;
pub use fixtures::*;
pub use gpu::detect_gpus;
pub use memory::detect_memory_info;
pub use platform::{detect_architecture, detect_os_info};
pub use profile::{
    AcceleratorCategory, Architecture, CpuInfo, DetectionConfidence, GpuInfo, GpuSnapshot,
    GpuVendor, HardwareProfile, HardwareSnapshot, HardwareSource, MemoryInfo, OperatingSystem,
};
pub use profiler::{HardwarePerformanceSnapshot, HardwareProfiler};
