use super::types::IpClassification;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Classifies an IP address for SSRF and network security boundary enforcement.
pub fn classify_ip(ip: IpAddr) -> IpClassification {
    match ip {
        IpAddr::V4(ipv4) => classify_ipv4(ipv4),
        IpAddr::V6(ipv6) => classify_ipv6(ipv6),
    }
}

/// Classifies an IPv4 address.
pub fn classify_ipv4(ip: Ipv4Addr) -> IpClassification {
    // Loopback: 127.0.0.0/8
    if ip.is_loopback() {
        return IpClassification::Loopback;
    }

    // Unspecified: 0.0.0.0
    if ip.is_unspecified() {
        return IpClassification::Unspecified;
    }

    // Broadcast: 255.255.255.255
    if ip.is_broadcast() {
        return IpClassification::Reserved;
    }

    // Multicast: 224.0.0.0/4
    if ip.is_multicast() {
        return IpClassification::Multicast;
    }

    // Link-local: 169.254.0.0/16 (includes cloud metadata 169.254.169.254)
    if ip.is_link_local() {
        return IpClassification::LinkLocal;
    }

    // Private IPv4 ranges:
    // 10.0.0.0/8
    // 172.16.0.0/12
    // 192.168.0.0/16
    // 100.64.0.0/10 (Carrier-grade NAT)
    // 192.0.0.0/24 (IETF Protocol Assignments)
    // 192.0.2.0/24 (TEST-NET-1)
    // 198.51.100.0/24 (TEST-NET-2)
    // 203.0.113.0/24 (TEST-NET-3)
    // 198.18.0.0/15 (Benchmarking)
    if ip.is_private() {
        return IpClassification::Private;
    }

    let octets = ip.octets();
    if octets[0] == 100 && (octets[1] & 0xc0) == 64 {
        return IpClassification::Private; // CGNAT
    }
    if octets[0] == 192 && octets[1] == 0 && octets[2] == 0 {
        return IpClassification::Reserved;
    }
    if octets[0] == 192 && octets[1] == 0 && octets[2] == 2 {
        return IpClassification::Reserved;
    }
    if octets[0] == 198 && octets[1] == 51 && octets[2] == 100 {
        return IpClassification::Reserved;
    }
    if octets[0] == 203 && octets[1] == 0 && octets[2] == 113 {
        return IpClassification::Reserved;
    }

    IpClassification::Public
}

/// Classifies an IPv6 address.
pub fn classify_ipv6(ip: Ipv6Addr) -> IpClassification {
    if ip.is_loopback() {
        return IpClassification::Loopback;
    }
    if ip.is_unspecified() {
        return IpClassification::Unspecified;
    }
    if ip.is_multicast() {
        return IpClassification::Multicast;
    }

    // Link-local unicast: fe80::/10
    let segments = ip.segments();
    if (segments[0] & 0xffc0) == 0xfe80 {
        return IpClassification::LinkLocal;
    }

    // Unique local address (IPv6 private): fc00::/7 (fc00:: and fd00::)
    if (segments[0] & 0xfe00) == 0xfc00 {
        return IpClassification::Private;
    }

    // IPv4-mapped IPv6 address: ::ffff:0:0/96
    if let Some(ipv4) = ip.to_ipv4_mapped() {
        return classify_ipv4(ipv4);
    }

    IpClassification::Public
}

/// Checks if a hostname or IP string represents a cloud metadata service endpoint.
pub fn is_cloud_metadata_endpoint(host: &str) -> bool {
    let lower = host.to_lowercase();
    let cleaned = lower.trim_matches(|c| c == '[' || c == ']');
    if cleaned == "169.254.169.254"
        || cleaned == "metadata.google.internal"
        || cleaned == "169.254.169.253"
        || cleaned.ends_with(".metadata.google.internal")
        || cleaned.ends_with(".internal")
    {
        return true;
    }
    false
}

trait EndsWithMetadata {
    fn ends_metadata(&self) -> bool;
}

impl EndsWithMetadata for str {
    fn ends_metadata(&self) -> bool {
        self.ends_with(".metadata.google.internal") || self == "metadata.google.internal"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_classification() {
        assert_eq!(
            classify_ip("127.0.0.1".parse().unwrap()),
            IpClassification::Loopback
        );
        assert_eq!(
            classify_ip("10.0.0.5".parse().unwrap()),
            IpClassification::Private
        );
        assert_eq!(
            classify_ip("172.16.0.1".parse().unwrap()),
            IpClassification::Private
        );
        assert_eq!(
            classify_ip("192.168.1.100".parse().unwrap()),
            IpClassification::Private
        );
        assert_eq!(
            classify_ip("169.254.169.254".parse().unwrap()),
            IpClassification::LinkLocal
        );
        assert_eq!(
            classify_ip("8.8.8.8".parse().unwrap()),
            IpClassification::Public
        );
        assert_eq!(
            classify_ip("::1".parse().unwrap()),
            IpClassification::Loopback
        );
        assert_eq!(
            classify_ip("fe80::1".parse().unwrap()),
            IpClassification::LinkLocal
        );
        assert_eq!(
            classify_ip("fd00::1".parse().unwrap()),
            IpClassification::Private
        );
    }

    #[test]
    fn test_metadata_endpoint_detection() {
        assert!(is_cloud_metadata_endpoint("169.254.169.254"));
        assert!(is_cloud_metadata_endpoint("metadata.google.internal"));
        assert!(!is_cloud_metadata_endpoint("api.openai.com"));
        assert!(!is_cloud_metadata_endpoint("localhost"));
    }
}
