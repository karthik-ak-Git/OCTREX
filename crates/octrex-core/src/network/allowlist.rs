use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowlistEntry {
    pub id: String,
    pub domain_pattern: String,
    pub description: String,
    pub added_at: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkAllowlist {
    pub entries: Vec<AllowlistEntry>,
}

impl NetworkAllowlist {
    pub fn new() -> Self {
        Self {
            entries: vec![
                AllowlistEntry {
                    id: "default-opencode".to_string(),
                    domain_pattern: "*.opencode.ai".to_string(),
                    description: "Default OpenCode router endpoints".to_string(),
                    added_at: 0,
                    enabled: true,
                },
                AllowlistEntry {
                    id: "default-openai".to_string(),
                    domain_pattern: "api.openai.com".to_string(),
                    description: "Approved OpenAI API endpoint".to_string(),
                    added_at: 0,
                    enabled: true,
                },
                AllowlistEntry {
                    id: "default-groq".to_string(),
                    domain_pattern: "api.groq.com".to_string(),
                    description: "Approved Groq API endpoint".to_string(),
                    added_at: 0,
                    enabled: true,
                },
                AllowlistEntry {
                    id: "default-nvidia".to_string(),
                    domain_pattern: "*.nvidia.com".to_string(),
                    description: "Approved NVIDIA NIM endpoints".to_string(),
                    added_at: 0,
                    enabled: true,
                },
                AllowlistEntry {
                    id: "default-google".to_string(),
                    domain_pattern: "*.googleapis.com".to_string(),
                    description: "Approved Google Gemini endpoints".to_string(),
                    added_at: 0,
                    enabled: true,
                },
            ],
        }
    }

    pub fn is_allowed(&self, host: &str) -> bool {
        let host_lower = host.to_lowercase();
        for entry in &self.entries {
            if !entry.enabled {
                continue;
            }
            let pattern_lower = entry.domain_pattern.to_lowercase();
            if pattern_lower == "*" {
                return true;
            }
            if let Some(suffix) = pattern_lower.strip_prefix("*.") {
                if host_lower == suffix || host_lower.ends_with(&format!(".{}", suffix)) {
                    return true;
                }
            } else if host_lower == pattern_lower {
                return true;
            }
        }
        false
    }

    pub fn add(
        &mut self,
        domain_pattern: impl Into<String>,
        description: impl Into<String>,
    ) -> AllowlistEntry {
        let entry = AllowlistEntry {
            id: format!("allow-{}", uuid::Uuid::new_v4()),
            domain_pattern: domain_pattern.into(),
            description: description.into(),
            added_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            enabled: true,
        };
        self.entries.push(entry.clone());
        entry
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let len_before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < len_before
    }
}
