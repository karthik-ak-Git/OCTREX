use crate::hardware::profile::{Architecture, OperatingSystem};
use sysinfo::System;

pub fn detect_os_info(_sys: &System) -> (OperatingSystem, Option<String>) {
    let os_name = System::name();
    let os_version = System::os_version();

    let detail = match (&os_name, &os_version) {
        (Some(n), Some(v)) => Some(format!("{} {}", n, v)),
        (Some(n), None) => Some(n.clone()),
        _ => None,
    };

    let os = if cfg!(target_os = "windows") {
        OperatingSystem::Windows
    } else if cfg!(target_os = "linux") {
        OperatingSystem::Linux
    } else if cfg!(target_os = "macos") {
        OperatingSystem::MacOs
    } else {
        OperatingSystem::Unknown(os_name.unwrap_or_else(|| "unknown".to_string()))
    };

    (os, detail)
}

pub fn detect_architecture() -> Architecture {
    match std::env::consts::ARCH {
        "x86_64" => Architecture::X86_64,
        "aarch64" => Architecture::Aarch64,
        "arm" => Architecture::Arm,
        "x86" => Architecture::X86,
        other => Architecture::Unknown(other.to_string()),
    }
}
