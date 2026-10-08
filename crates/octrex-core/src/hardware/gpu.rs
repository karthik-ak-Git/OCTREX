use super::profile::{AcceleratorCategory, GpuInfo, GpuVendor};
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

pub async fn detect_gpus() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    // 1. Try NVIDIA detection via nvidia-smi
    if let Ok(nvidia_gpus) = detect_nvidia_smi().await {
        if !nvidia_gpus.is_empty() {
            gpus.extend(nvidia_gpus);
            return gpus;
        }
    }

    // 2. Apple Silicon / macOS unified memory detection
    #[cfg(target_os = "macos")]
    {
        if let Some(apple_gpu) = detect_apple_silicon_gpu() {
            gpus.push(apple_gpu);
            return gpus;
        }
    }

    // 3. Platform fallback (Windows / Linux GPU search via OS interrogation)
    #[cfg(target_os = "windows")]
    {
        if let Ok(win_gpus) = detect_windows_gpus().await {
            gpus.extend(win_gpus);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(linux_gpus) = detect_linux_gpus().await {
            gpus.extend(linux_gpus);
        }
    }

    gpus
}

/// Safely execute `nvidia-smi` without shell invocation.
async fn detect_nvidia_smi() -> Result<Vec<GpuInfo>, String> {
    let res = timeout(
        Duration::from_secs(2),
        tokio::task::spawn_blocking(|| {
            let output = Command::new("nvidia-smi")
                .args([
                    "--query-gpu=index,name,memory.total,memory.free,driver_version",
                    "--format=csv,noheader,nounits",
                ])
                .output();
            output
        }),
    )
    .await;

    let output = match res {
        Ok(Ok(Ok(out))) => out,
        _ => return Err("nvidia-smi not available or timed out".to_string()),
    };

    if !output.status.success() {
        return Err("nvidia-smi returned non-zero exit code".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 5 {
            let index: u32 = parts[0].parse().unwrap_or(0);
            let name = parts[1].to_string();
            let total_mb: Option<u64> = parts[2].parse().ok();
            let free_mb: Option<u64> = parts[3].parse().ok();
            let driver_version = if !parts[4].is_empty() {
                Some(parts[4].to_string())
            } else {
                None
            };

            let vram_bytes = total_mb.map(|mb| mb * 1024 * 1024);
            let available_vram_bytes = free_mb.map(|mb| mb * 1024 * 1024);

            gpus.push(GpuInfo {
                vendor: GpuVendor::Nvidia,
                name,
                vram_bytes,
                available_vram_bytes,
                driver_version,
                device_index: index,
                accelerator_category: AcceleratorCategory::DiscreteGpu,
            });
        }
    }

    Ok(gpus)
}

#[cfg(target_os = "macos")]
fn detect_apple_silicon_gpu() -> Option<GpuInfo> {
    if cfg!(target_arch = "aarch64") {
        let sys = sysinfo::System::new_all();
        let total_ram = sys.total_memory();
        Some(GpuInfo {
            vendor: GpuVendor::Apple,
            name: "Apple Silicon GPU (Unified Memory)".to_string(),
            vram_bytes: Some(total_ram),
            available_vram_bytes: Some(sys.available_memory()),
            driver_version: None,
            device_index: 0,
            accelerator_category: AcceleratorCategory::UnifiedMemory,
        })
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
async fn detect_windows_gpus() -> Result<Vec<GpuInfo>, String> {
    // Windows fallback using wmic or powershell for display adapter info if nvidia-smi missed it
    let res = timeout(
        Duration::from_secs(2),
        tokio::task::spawn_blocking(|| {
            Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "Get-CimInstance Win32_VideoController | Select-Name, AdapterRAM, DriverVersion | ConvertTo-Json",
                ])
                .output()
        }),
    )
    .await;

    let output = match res {
        Ok(Ok(Ok(out))) if out.status.success() => out,
        _ => return Ok(Vec::new()),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();

    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let items = if val.is_array() {
            val.as_array().cloned().unwrap_or_default()
        } else if val.is_object() {
            vec![val]
        } else {
            vec![]
        };

        for (idx, item) in items.iter().enumerate() {
            if let Some(name) = item.get("Name").and_then(|v| v.as_str()) {
                let lower_name = name.to_lowercase();
                let vendor = if lower_name.contains("nvidia") {
                    GpuVendor::Nvidia
                } else if lower_name.contains("amd") || lower_name.contains("radeon") {
                    GpuVendor::Amd
                } else if lower_name.contains("intel") {
                    GpuVendor::Intel
                } else {
                    GpuVendor::Unknown(name.to_string())
                };

                let adapter_ram = item.get("AdapterRAM").and_then(|v| v.as_u64());
                let driver_version = item
                    .get("DriverVersion")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let category = if vendor == GpuVendor::Intel
                    && (adapter_ram.unwrap_or(0) < 1024 * 1024 * 1024)
                {
                    AcceleratorCategory::IntegratedGpu
                } else {
                    AcceleratorCategory::DiscreteGpu
                };

                gpus.push(GpuInfo {
                    vendor,
                    name: name.to_string(),
                    vram_bytes: adapter_ram.filter(|&r| r > 0),
                    available_vram_bytes: None, // free vram unknown via CIM
                    driver_version,
                    device_index: idx as u32,
                    accelerator_category: category,
                });
            }
        }
    }

    Ok(gpus)
}

#[cfg(target_os = "linux")]
async fn detect_linux_gpus() -> Result<Vec<GpuInfo>, String> {
    // Linux fallback using /sys/class/drm
    let mut gpus = Vec::new();
    let drm_path = std::path::Path::new("/sys/class/drm");
    if drm_path.exists() {
        if let Ok(entries) = std::fs::read_dir(drm_path) {
            let mut idx = 0;
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("card") && !name.contains('-') {
                    gpus.push(GpuInfo {
                        vendor: GpuVendor::Unknown("Linux DRM Device".to_string()),
                        name: format!("Linux DRM {}", name),
                        vram_bytes: None,
                        available_vram_bytes: None,
                        driver_version: None,
                        device_index: idx,
                        accelerator_category: AcceleratorCategory::Unknown,
                    });
                    idx += 1;
                }
            }
        }
    }
    Ok(gpus)
}
