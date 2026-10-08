use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize, PartialEq, Eq)]
pub enum HardwareError {
    #[error("Hardware detection failed: {message}")]
    DetectionFailed { message: String },

    #[error("GPU detection failed: {message}")]
    GpuDetectionFailed { message: String },

    #[error("Memory detection failed: {message}")]
    MemoryDetectionFailed { message: String },

    #[error("Platform unsupported: {platform}")]
    UnsupportedPlatform { platform: String },

    #[error("Hardware information unavailable: {reason}")]
    InformationUnavailable { reason: String },

    #[error("Subprocess execution failed or timed out: {command}")]
    SubprocessFailed { command: String },
}
