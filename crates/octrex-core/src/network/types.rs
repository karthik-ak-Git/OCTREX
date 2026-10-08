use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMode {
    #[default]
    LocalOnly,
    Restricted,
    OnlineAllowed,
    Disabled,
}

impl fmt::Display for NetworkMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalOnly => write!(f, "local_only"),
            Self::Restricted => write!(f, "restricted"),
            Self::OnlineAllowed => write!(f, "online_allowed"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkProtocol {
    Http,
    Https,
    WebSocket,
    Tcp,
}

impl fmt::Display for NetworkProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http => write!(f, "http"),
            Self::Https => write!(f, "https"),
            Self::WebSocket => write!(f, "ws"),
            Self::Tcp => write!(f, "tcp"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkCapability {
    ExternalHttps,
    ExternalHttp,
    CloudModelInference,
    WebFetch,
    WebSearch,
    RemoteMcp,
    ProviderApi,
    LocalNetwork,
    Loopback,
}

impl fmt::Display for NetworkCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExternalHttps => write!(f, "external_https"),
            Self::ExternalHttp => write!(f, "external_http"),
            Self::CloudModelInference => write!(f, "cloud_model_inference"),
            Self::WebFetch => write!(f, "web_fetch"),
            Self::WebSearch => write!(f, "web_search"),
            Self::RemoteMcp => write!(f, "remote_mcp"),
            Self::ProviderApi => write!(f, "provider_api"),
            Self::LocalNetwork => write!(f, "local_network"),
            Self::Loopback => write!(f, "loopback"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpClassification {
    Public,
    Private,
    Loopback,
    LinkLocal,
    Multicast,
    Reserved,
    Unspecified,
    Unknown,
}

impl IpClassification {
    pub fn is_safe_for_external(&self) -> bool {
        matches!(self, Self::Public)
    }

    pub fn is_loopback(&self) -> bool {
        matches!(self, Self::Loopback)
    }

    pub fn is_local_network(&self) -> bool {
        matches!(self, Self::Loopback | Self::Private | Self::LinkLocal)
    }
}

impl fmt::Display for IpClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Private => write!(f, "private"),
            Self::Loopback => write!(f, "loopback"),
            Self::LinkLocal => write!(f, "link_local"),
            Self::Multicast => write!(f, "multicast"),
            Self::Reserved => write!(f, "reserved"),
            Self::Unspecified => write!(f, "unspecified"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
