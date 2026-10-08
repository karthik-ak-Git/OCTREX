use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident, $prefix:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new() -> Self {
                Self(format!("{}-{}", $prefix, Uuid::new_v4().simple()))
            }

            pub fn from_string(s: impl Into<String>) -> Self {
                let s = s.into();
                if s.starts_with($prefix) {
                    Self(s)
                } else {
                    Self(format!("{}-{}", $prefix, s))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = std::convert::Infallible;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self::from_string(s))
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self::from_string(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self::from_string(s)
            }
        }
    };
}

define_id!(RequestId, "req", "Strongly typed Request Identifier");
define_id!(TaskId, "task", "Strongly typed Task Identifier");
define_id!(SessionId, "session", "Strongly typed Session Identifier");
define_id!(WorkspaceId, "ws", "Strongly typed Workspace Identifier");
define_id!(EventId, "evt", "Strongly typed Event Identifier");
define_id!(ToolCallId, "tool", "Strongly typed Tool Call Identifier");
define_id!(ModelCallId, "model", "Strongly typed Model Call Identifier");
define_id!(ArtifactId, "art", "Strongly typed Artifact Identifier");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_prefix_and_formatting() {
        let task_id = TaskId::new();
        assert!(task_id.as_str().starts_with("task-"));

        let session_id = SessionId::from_string("custom-123");
        assert_eq!(session_id.as_str(), "session-custom-123");

        let req_id = RequestId::from_string("req-already-prefixed");
        assert_eq!(req_id.as_str(), "req-already-prefixed");
    }
}
