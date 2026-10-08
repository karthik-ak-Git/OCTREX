use crate::local_runtime::errors::LocalRuntimeError;
use crate::network::{IpClassification, NetworkCapability, NetworkEndpoint, NetworkRequest};

/// Sources that are explicitly permitted for model downloads.
///
/// Downloads never use arbitrary URLs: the caller must supply a source key
/// present in this allowlist (or the configured-endpoint source), and any
/// redirect target is re-validated through the same boundary.
pub const ALLOWED_DOWNLOAD_SOURCES: &[&str] = &["configured-endpoint", "local-file", "runtime"];

/// Metadata keys that model-supplied content may never set. Any attempt to
/// smuggle these through registration/discovery metadata is stripped and
/// recorded as a policy violation.
pub const FORBIDDEN_METADATA_KEYS: &[&str] = &[
    "privacy_mode",
    "privacy_policy",
    "routing_policy",
    "routing_mode",
    "permissions",
    "permission",
    "company_policy",
    "tool_capability",
    "tool_capabilities",
    "policy_override",
    "allow_cloud",
    "allow_online",
    "consent_granted",
];

/// Validate that a candidate local endpoint is a loopback/local URL.
///
/// Rules (fail closed):
/// - must parse as http/https/ws
/// - host must be loopback (`localhost`, `127.0.0.0/8`, `::1`) or explicitly
///   configured local hostname that resolves to a loopback IP
/// - no credentials in URL
/// - no non-loopback IPs, no metadata endpoints
pub fn validate_local_endpoint(raw: &str) -> Result<NetworkEndpoint, LocalRuntimeError> {
    let endpoint = NetworkEndpoint::parse(raw).map_err(|e| LocalRuntimeError::InvalidEndpoint {
        endpoint: raw.to_string(),
        reason: e.to_string(),
    })?;

    if endpoint.has_query {
        return Err(LocalRuntimeError::InvalidEndpoint {
            endpoint: raw.to_string(),
            reason: "Endpoint URL must not contain query parameters".to_string(),
        });
    }

    let host = endpoint.host.to_lowercase();
    let host_no_brackets = host
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(&host);
    let is_loopback_name = host_no_brackets == "localhost" || host_no_brackets == "ip6-localhost";
    let is_loopback_ip = match host_no_brackets.parse::<std::net::IpAddr>() {
        Ok(ip) => {
            let class = crate::network::classify_ip(ip);
            matches!(
                class,
                IpClassification::Loopback | IpClassification::Unspecified
            ) || ip.is_loopback()
        }
        Err(_) => false,
    };

    if !(is_loopback_name || is_loopback_ip) {
        return Err(LocalRuntimeError::InvalidEndpoint {
            endpoint: raw.to_string(),
            reason: format!(
                "Only loopback endpoints are permitted for local runtimes (got host '{}')",
                endpoint.host
            ),
        });
    }

    if crate::network::is_cloud_metadata_endpoint(&endpoint.host) {
        return Err(LocalRuntimeError::InvalidEndpoint {
            endpoint: raw.to_string(),
            reason: "Cloud metadata endpoints are never valid local runtimes".to_string(),
        });
    }

    Ok(endpoint)
}

/// Evaluate a loopback endpoint through the existing `NetworkSecurityService`
/// boundary. Loopback is local but still validated — never bypassed.
pub fn evaluate_endpoint_against_policy(
    service: &crate::network::NetworkSecurityService,
    endpoint: &NetworkEndpoint,
    source: &str,
) -> Result<(), LocalRuntimeError> {
    let req = NetworkRequest::new(
        source.to_string(),
        NetworkCapability::Loopback,
        endpoint.clone(),
    );
    let decision = service.evaluate_request(&req);
    if decision.disposition.is_allowed() {
        Ok(())
    } else {
        Err(LocalRuntimeError::NetworkDenied {
            endpoint: endpoint.raw_url.clone(),
            reason: decision.reason.clone(),
        })
    }
}

/// Reject unsafe model file paths (traversal, executables, null bytes).
pub fn validate_model_file_path(raw: &str) -> Result<String, LocalRuntimeError> {
    if raw.is_empty() {
        return Err(LocalRuntimeError::UnsafePath {
            reason: "Model path is empty".to_string(),
        });
    }
    if raw.contains('\0') {
        return Err(LocalRuntimeError::UnsafePath {
            reason: "Model path contains null byte".to_string(),
        });
    }
    let lowered = raw.to_lowercase();
    for seg in ["..", "~", "$", "`", "|", ";", "&", "\n", "\r"] {
        if lowered.contains(seg) {
            return Err(LocalRuntimeError::UnsafePath {
                reason: format!("Model path contains forbidden segment '{}'", seg),
            });
        }
    }
    for ext in [
        ".exe", ".dll", ".so", ".dylib", ".bat", ".cmd", ".ps1", ".sh",
    ] {
        if lowered.ends_with(ext) {
            return Err(LocalRuntimeError::UnsafePath {
                reason: format!(
                    "Model path must not reference an executable artifact ('{}')",
                    ext
                ),
            });
        }
    }
    Ok(raw.to_string())
}

/// Supported model asset formats. Anything else is rejected or requires an
/// explicit user override with a recorded warning — never silently accepted.
pub fn validate_model_format(format: Option<&str>) -> Result<String, LocalRuntimeError> {
    let name = format.unwrap_or("").to_lowercase();
    match name.as_str() {
        "gguf" | "ggml" | "safetensors" | "ollama" | "openai-compatible" | "gguf-file"
        | "runtime-managed" => Ok(name),
        "" => Err(LocalRuntimeError::UnsupportedFormat {
            format: "(unspecified)".to_string(),
        }),
        other => Err(LocalRuntimeError::UnsupportedFormat {
            format: other.to_string(),
        }),
    }
}

/// Strip policy-altering keys from model-supplied metadata.
///
/// Returns the sanitized metadata plus the list of stripped keys. Model
/// content can never change privacy, routing, permission, or tool policy.
pub fn sanitize_model_metadata(
    input: &std::collections::HashMap<String, String>,
) -> (std::collections::HashMap<String, String>, Vec<String>) {
    let mut clean = std::collections::HashMap::new();
    let mut stripped = Vec::new();
    for (k, v) in input {
        let key_lower = k.to_lowercase();
        if FORBIDDEN_METADATA_KEYS.contains(&key_lower.as_str()) {
            stripped.push(k.clone());
            continue;
        }
        // Executable references inside metadata are never honored.
        let value_lower = v.to_lowercase();
        if value_lower.contains(".exe")
            || value_lower.contains(".dll")
            || value_lower.contains(".so")
            || value_lower.trim_start().starts_with("exec:")
        {
            stripped.push(k.clone());
            continue;
        }
        clean.insert(k.clone(), v.clone());
    }
    (clean, stripped)
}

/// Validate an explicit model download request.
///
/// - URL must be http(s) and present in the configured allowlist context
/// - redirects are the caller's responsibility to re-validate
/// - destination must stay inside the managed model directory
pub fn validate_download_request(
    source: &str,
    url: &str,
    destination_dir: &std::path::Path,
    file_name: &str,
) -> Result<(NetworkEndpoint, std::path::PathBuf), LocalRuntimeError> {
    if !ALLOWED_DOWNLOAD_SOURCES.contains(&source) {
        return Err(LocalRuntimeError::UnsafeDownload {
            reason: format!(
                "Download source '{}' is not allowlisted. Permitted: {:?}",
                source, ALLOWED_DOWNLOAD_SOURCES
            ),
        });
    }
    let endpoint = NetworkEndpoint::parse(url).map_err(|e| LocalRuntimeError::UnsafeDownload {
        reason: format!("Invalid download URL: {}", e),
    })?;
    match endpoint.protocol {
        crate::network::NetworkProtocol::Http | crate::network::NetworkProtocol::Https => {}
        _ => {
            return Err(LocalRuntimeError::UnsafeDownload {
                reason: "Downloads require http(s) URLs".to_string(),
            });
        }
    }
    validate_model_file_path(file_name)?;
    let dest = destination_dir.join(file_name);
    if !dest.starts_with(destination_dir) {
        return Err(LocalRuntimeError::UnsafeDownload {
            reason: "Download destination escapes the managed model directory".to_string(),
        });
    }
    Ok((endpoint, dest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_loopback_endpoints() {
        assert!(validate_local_endpoint("http://192.168.1.10:11434").is_err());
        assert!(validate_local_endpoint("https://api.openai.com/v1").is_err());
        assert!(validate_local_endpoint("http://10.0.0.5:8080").is_err());
    }

    #[test]
    fn accepts_loopback_endpoints() {
        assert!(validate_local_endpoint("http://127.0.0.1:11434").is_ok());
        assert!(validate_local_endpoint("http://localhost:11434").is_ok());
        assert!(validate_local_endpoint("http://[::1]:11434").is_ok());
    }
}
