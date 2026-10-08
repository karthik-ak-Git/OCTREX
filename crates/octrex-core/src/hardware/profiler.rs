use crate::hardware::detector::HardwareService;
use crate::hardware::errors::HardwareError;
use crate::hardware::profile::HardwareSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwarePerformanceSnapshot {
    pub timestamp: u64,
    pub ram_total_bytes: u64,
    pub ram_available_bytes: u64,
    pub ram_usage_percent: f32,
    pub cpu_usage_percent: f32,
    pub gpu_count: usize,
    pub active_accelerators: Vec<String>,
}

pub struct HardwareProfiler;

impl HardwareProfiler {
    pub async fn capture_snapshot(
        service: &HardwareService,
    ) -> Result<HardwarePerformanceSnapshot, HardwareError> {
        let snap: HardwareSnapshot = service.snapshot().await?;
        let active_accelerators = snap
            .gpu_snapshots
            .iter()
            .map(|g| format!("GPU {}: {}", g.device_index, g.name))
            .collect();

        Ok(HardwarePerformanceSnapshot {
            timestamp: snap.timestamp,
            ram_total_bytes: snap.total_ram_bytes,
            ram_available_bytes: snap.available_ram_bytes,
            ram_usage_percent: snap.ram_usage_percent,
            cpu_usage_percent: snap.cpu_usage_percent,
            gpu_count: snap.gpu_snapshots.len(),
            active_accelerators,
        })
    }
}
