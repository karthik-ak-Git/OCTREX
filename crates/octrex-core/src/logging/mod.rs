use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String, // "pretty" or "json"
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
        }
    }
}

pub fn init_logging(_config: &LoggingConfig) {
    // Tracing subscriber initialization is safe to call once.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("octrex=info".parse().unwrap_or_default()),
        )
        .try_init();
}

/// Mask secret strings so they are never printed in log output
pub fn mask_secret(secret: &str) -> String {
    if secret.is_empty() {
        return String::new();
    }
    if secret.len() <= 6 {
        return "******".to_string();
    }
    format!("{}...{}", &secret[..3], &secret[secret.len() - 3..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_secret() {
        assert_eq!(mask_secret("gsk_1234567890abcdef"), "gsk...def");
        assert_eq!(mask_secret("123"), "******");
    }
}
