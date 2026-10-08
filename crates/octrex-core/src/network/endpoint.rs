use super::errors::NetworkError;
use super::types::NetworkProtocol;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkEndpoint {
    pub raw_url: String,
    pub protocol: NetworkProtocol,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub has_query: bool,
    pub resolved_ips: Vec<IpAddr>,
}

impl NetworkEndpoint {
    /// Parses and normalizes a raw URL into a NetworkEndpoint.
    pub fn parse(raw: &str) -> Result<Self, NetworkError> {
        let parsed = Url::parse(raw).map_err(|e| NetworkError::InvalidEndpoint {
            message: format!("URL parse error: {}", e),
        })?;

        let scheme = parsed.scheme().to_lowercase();
        let protocol = match scheme.as_str() {
            "http" => NetworkProtocol::Http,
            "https" => NetworkProtocol::Https,
            "ws" | "wss" => NetworkProtocol::WebSocket,
            "tcp" => NetworkProtocol::Tcp,
            unsupported => {
                return Err(NetworkError::UnsupportedProtocol {
                    protocol: unsupported.to_string(),
                });
            }
        };

        let host = parsed
            .host_str()
            .ok_or_else(|| NetworkError::InvalidEndpoint {
                message: "Missing hostname in URL".to_string(),
            })?
            .to_lowercase();

        let port = parsed.port_or_known_default().unwrap_or(match protocol {
            NetworkProtocol::Http => 80,
            NetworkProtocol::Https => 443,
            NetworkProtocol::WebSocket => 80,
            NetworkProtocol::Tcp => 80,
        });

        let path = parsed.path().to_string();
        let has_query = parsed.query().is_some();

        // Sanitize raw URL so secrets in query params are not preserved
        let mut sanitized_url = parsed.clone();
        sanitized_url.set_query(None);
        sanitized_url.set_password(None).ok();
        sanitized_url.set_username("").ok();

        Ok(Self {
            raw_url: sanitized_url.to_string(),
            protocol,
            host,
            port,
            path,
            has_query,
            resolved_ips: Vec::new(),
        })
    }

    /// Returns a clean canonical host:port identifier (e.g., "api.openai.com:443")
    pub fn host_and_port(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Checks if this endpoint matches a domain pattern (e.g. "*.openai.com" or "api.openai.com")
    pub fn matches_domain_pattern(&self, pattern: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();
        if pattern_lower == "*" {
            return true;
        }

        if let Some(suffix) = pattern_lower.strip_prefix("*.") {
            self.host == suffix || self.host.ends_with(&format!(".{}", suffix))
        } else {
            self.host == pattern_lower
        }
    }
}

impl fmt::Display for NetworkEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}:{}", self.protocol, self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_parsing_and_normalization() {
        let ep = NetworkEndpoint::parse("HTTPS://API.EXAMPLE.COM:443/v1/chat?api_key=secret123")
            .unwrap();
        assert_eq!(ep.protocol, NetworkProtocol::Https);
        assert_eq!(ep.host, "api.example.com");
        assert_eq!(ep.port, 443);
        assert_eq!(ep.path, "/v1/chat");
        assert!(ep.has_query);
        assert!(!ep.raw_url.contains("secret123")); // Secrets redacted from raw_url!
    }

    #[test]
    fn test_domain_pattern_matching() {
        let ep = NetworkEndpoint::parse("https://api.openai.com/v1/models").unwrap();
        assert!(ep.matches_domain_pattern("api.openai.com"));
        assert!(ep.matches_domain_pattern("*.openai.com"));
        assert!(ep.matches_domain_pattern("*"));
        assert!(!ep.matches_domain_pattern("api.anthropic.com"));
    }
}
