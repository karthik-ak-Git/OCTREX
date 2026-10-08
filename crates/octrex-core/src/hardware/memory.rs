use crate::hardware::profile::MemoryInfo;
use sysinfo::System;

pub fn detect_memory_info(sys: &System) -> MemoryInfo {
    let total_bytes = sys.total_memory();
    let available_bytes = Some(sys.available_memory());
    let used_bytes = Some(sys.used_memory());

    let is_unified_memory = cfg!(target_os = "macos") && cfg!(target_arch = "aarch64");

    MemoryInfo {
        total_bytes,
        available_bytes,
        used_bytes,
        is_unified_memory,
    }
}
