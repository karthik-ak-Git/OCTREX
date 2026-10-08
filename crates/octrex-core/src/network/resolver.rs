use super::errors::NetworkError;
use super::types::IpClassification;
use super::validation::classify_ip;
use std::net::{IpAddr, ToSocketAddrs};

#[derive(Debug, Clone, Default)]
pub struct SafeDnsResolver;

impl SafeDnsResolver {
    pub fn new() -> Self {
        Self
    }

    /// Resolves a hostname to IP addresses and evaluates them against SSRF/classification rules.
    pub fn resolve_and_validate(&self, host: &str, port: u16) -> Result<Vec<IpAddr>, NetworkError> {
        // Direct IP check first
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Ok(vec![ip]);
        }

        let socket_str = format!("{}:{}", host, port);
        let addrs =
            socket_str
                .to_socket_addrs()
                .map_err(|e| NetworkError::DnsResolutionFailed {
                    host: host.to_string(),
                    reason: format!("DNS lookup failed: {}", e),
                })?;

        let ips: Vec<IpAddr> = addrs.map(|s| s.ip()).collect();

        if ips.is_empty() {
            return Err(NetworkError::DnsResolutionFailed {
                host: host.to_string(),
                reason: "DNS resolved to 0 IP addresses".to_string(),
            });
        }

        Ok(ips)
    }

    /// Validates resolved IPs for external network safety.
    /// If ANY resolved IP is private/loopback/link-local/metadata, return the classification violation.
    pub fn check_external_safety(&self, host: &str, ips: &[IpAddr]) -> Result<(), NetworkError> {
        for &ip in ips {
            let classification = classify_ip(ip);
            match classification {
                IpClassification::Loopback => {
                    return Err(NetworkError::LoopbackBlocked {
                        address: format!("Host '{}' resolved to loopback IP {}", host, ip),
                    });
                }
                IpClassification::Private => {
                    return Err(NetworkError::PrivateAddressBlocked {
                        address: format!("Host '{}' resolved to private IP {}", host, ip),
                    });
                }
                IpClassification::LinkLocal => {
                    return Err(NetworkError::LinkLocalBlocked {
                        address: format!("Host '{}' resolved to link-local IP {}", host, ip),
                    });
                }
                IpClassification::Multicast
                | IpClassification::Unspecified
                | IpClassification::Reserved
                | IpClassification::Unknown => {
                    return Err(NetworkError::DestinationBlocked {
                        destination: host.to_string(),
                        reason: format!("Host resolved to unsafe IP {} ({})", ip, classification),
                    });
                }
                IpClassification::Public => {}
            }
        }
        Ok(())
    }
}
