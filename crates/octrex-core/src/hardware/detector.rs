use crate::events::{EventEnvelope, EventType};
use crate::hardware::compatibility::{CompatibilityResult, ModelCompatibilityEngine};
use crate::hardware::cpu::detect_cpu_info;
use crate::hardware::errors::HardwareError;
use crate::hardware::gpu::detect_gpus;
use crate::hardware::memory::detect_memory_info;
use crate::hardware::platform::{detect_architecture, detect_os_info};
use crate::hardware::profile::{
    DetectionConfidence, GpuSnapshot, HardwareProfile, HardwareSnapshot, HardwareSource,
};
use crate::models::types::ModelDescriptor;
use async_trait::async_trait;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

#[async_trait]
pub trait HardwareProbe: Send + Sync {
    async fn detect_profile(&self) -> Result<HardwareProfile, HardwareError>;
    async fn detect_snapshot(&self) -> Result<HardwareSnapshot, HardwareError>;
}

pub struct RealHardwareProbe;

#[async_trait]
impl HardwareProbe for RealHardwareProbe {
    async fn detect_profile(&self) -> Result<HardwareProfile, HardwareError> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let (os, os_detail) = detect_os_info(&sys);
        let arch = detect_architecture();
        let cpu = detect_cpu_info(&sys);
        let memory = detect_memory_info(&sys);

        let gpus = detect_gpus().await;

        let confidence = if !gpus.is_empty()
            && gpus.iter().any(|g| g.vram_bytes.is_some())
            && memory.total_bytes > 0
        {
            DetectionConfidence::Exact
        } else if memory.total_bytes > 0 {
            DetectionConfidence::High
        } else {
            DetectionConfidence::Estimated
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(HardwareProfile {
            os,
            os_detail,
            arch,
            cpu,
            memory,
            gpus,
            detected_at_timestamp: timestamp,
            confidence,
            source: HardwareSource::SystemApi,
        })
    }

    async fn detect_snapshot(&self) -> Result<HardwareSnapshot, HardwareError> {
        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.refresh_cpu();

        let total_ram_bytes = sys.total_memory();
        let available_ram_bytes = sys.available_memory();
        let ram_usage_percent = if total_ram_bytes > 0 {
            ((total_ram_bytes.saturating_sub(available_ram_bytes)) as f32 / total_ram_bytes as f32)
                * 100.0
        } else {
            0.0
        };

        let cpu_usage_percent = sys.global_cpu_info().cpu_usage();

        let gpus = detect_gpus().await;
        let gpu_snapshots = gpus
            .into_iter()
            .map(|g| GpuSnapshot {
                device_index: g.device_index,
                name: g.name,
                vram_used_bytes: match (g.vram_bytes, g.available_vram_bytes) {
                    (Some(t), Some(a)) => Some(t.saturating_sub(a)),
                    _ => None,
                },
                vram_available_bytes: g.available_vram_bytes,
                utilization_percent: None,
                temperature_celsius: None,
            })
            .collect();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(HardwareSnapshot {
            timestamp,
            total_ram_bytes,
            available_ram_bytes,
            ram_usage_percent,
            cpu_usage_percent,
            gpu_snapshots,
        })
    }
}

pub struct MockHardwareProbe {
    pub profile: HardwareProfile,
    pub snapshot: HardwareSnapshot,
}

impl MockHardwareProbe {
    pub fn new(profile: HardwareProfile, snapshot: HardwareSnapshot) -> Self {
        Self { profile, snapshot }
    }
}

#[async_trait]
impl HardwareProbe for MockHardwareProbe {
    async fn detect_profile(&self) -> Result<HardwareProfile, HardwareError> {
        Ok(self.profile.clone())
    }

    async fn detect_snapshot(&self) -> Result<HardwareSnapshot, HardwareError> {
        Ok(self.snapshot.clone())
    }
}

pub struct HardwareService {
    probe: Arc<dyn HardwareProbe>,
    cached_profile: RwLock<Option<HardwareProfile>>,
    cached_snapshot: RwLock<Option<HardwareSnapshot>>,
    event_bus: Option<Arc<crate::events::EventBus>>,
}

impl HardwareService {
    pub fn new(probe: Arc<dyn HardwareProbe>) -> Self {
        Self {
            probe,
            cached_profile: RwLock::new(None),
            cached_snapshot: RwLock::new(None),
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: Arc<crate::events::EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub fn real() -> Self {
        Self::new(Arc::new(RealHardwareProbe))
    }

    pub async fn detect(&self) -> Result<HardwareProfile, HardwareError> {
        if let Some(bus) = &self.event_bus {
            let _ = bus.publish(EventEnvelope::new(
                EventType::HardwareDetectionStarted,
                serde_json::json!({ "status": "STARTED" }),
            ));
        }

        match self.probe.detect_profile().await {
            Ok(profile) => {
                {
                    let mut guard = self.cached_profile.write().unwrap();
                    *guard = Some(profile.clone());
                }

                if let Some(bus) = &self.event_bus {
                    let _ = bus.publish(EventEnvelope::new(
                        EventType::HardwareDetectionCompleted,
                        serde_json::json!({
                            "os": profile.os.to_string(),
                            "cpu_cores": profile.cpu.logical_cores,
                            "ram_mb": profile.memory.total_ram_mb(),
                            "gpu_count": profile.gpus.len(),
                            "confidence": profile.confidence.to_string()
                        }),
                    ));
                    let _ = bus.publish(EventEnvelope::new(
                        EventType::HardwareProfileChanged,
                        serde_json::json!({ "timestamp": profile.detected_at_timestamp }),
                    ));
                }

                Ok(profile)
            }
            Err(err) => {
                if let Some(bus) = &self.event_bus {
                    let _ = bus.publish(EventEnvelope::new(
                        EventType::HardwareDetectionFailed,
                        serde_json::json!({ "error": err.to_string() }),
                    ));
                }
                Err(err)
            }
        }
    }

    pub fn get_profile(&self) -> HardwareProfile {
        {
            let guard = self.cached_profile.read().unwrap();
            if let Some(p) = guard.as_ref() {
                return p.clone();
            }
        }
        HardwareProfile::default()
    }

    pub async fn profile(&self) -> Result<HardwareProfile, HardwareError> {
        {
            let guard = self.cached_profile.read().unwrap();
            if let Some(p) = guard.as_ref() {
                return Ok(p.clone());
            }
        }
        self.detect().await
    }

    pub async fn refresh(&self) -> Result<HardwareProfile, HardwareError> {
        {
            let mut guard = self.cached_profile.write().unwrap();
            *guard = None;
        }
        self.detect().await
    }

    pub async fn snapshot(&self) -> Result<HardwareSnapshot, HardwareError> {
        let snap = self.probe.detect_snapshot().await?;
        {
            let mut guard = self.cached_snapshot.write().unwrap();
            *guard = Some(snap.clone());
        }
        Ok(snap)
    }

    pub fn evaluate_compatibility(
        &self,
        profile: &HardwareProfile,
        model: &ModelDescriptor,
    ) -> CompatibilityResult {
        let result = ModelCompatibilityEngine::evaluate(profile, model);
        if let Some(bus) = &self.event_bus {
            let _ = bus.publish(EventEnvelope::new(
                EventType::ModelCompatibilityChecked,
                serde_json::json!({
                    "model_id": result.model_id,
                    "status": result.status,
                    "suitable_devices": result.suitable_devices
                }),
            ));
        }
        result
    }
}
