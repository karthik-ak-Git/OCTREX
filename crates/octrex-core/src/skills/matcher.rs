use crate::ids::{TaskId, WorkspaceId};
use crate::skills::policy;
use crate::skills::registry::SkillRegistry;
use crate::skills::types::SkillDefinition;

/// Ranked skill candidate. Security eligibility is evaluated first; only
/// eligible skills are ranked.
#[derive(Debug, Clone)]
pub struct RankedSkill {
    pub skill: SkillDefinition,
    pub score: f64,
    pub reasons: Vec<String>,
}

pub struct SkillMatchRequest {
    pub user_request: String,
    pub task_id: Option<TaskId>,
    pub workspace_id: Option<WorkspaceId>,
    pub task_type: Option<String>,
    pub limit: usize,
}

impl Default for SkillMatchRequest {
    fn default() -> Self {
        Self {
            user_request: String::new(),
            task_id: None,
            workspace_id: None,
            task_type: None,
            limit: 5,
        }
    }
}

pub struct SkillMatcher;

impl SkillMatcher {
    pub fn match_skills(registry: &SkillRegistry, req: &SkillMatchRequest) -> Vec<RankedSkill> {
        let mut out = Vec::new();
        let request_lower = req.user_request.to_lowercase();
        let request_tokens: Vec<&str> = request_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() > 2)
            .collect();

        for skill in registry.list() {
            // Security eligibility comes first; ineligible skills never match.
            if policy::check_eligibility(&skill).is_err() {
                continue;
            }
            if policy::check_policy_compatibility(&skill).is_err() {
                continue;
            }

            let mut score = 0.0;
            let mut reasons = Vec::new();

            let haystack = format!(
                "{} {} {}",
                skill.name.to_lowercase(),
                skill.description.to_lowercase(),
                skill.allowed_tools.join(" ").to_lowercase()
            );
            let mut hits = 0;
            for tok in &request_tokens {
                if haystack.contains(tok) {
                    hits += 1;
                }
            }
            if !request_tokens.is_empty() {
                let intent = hits as f64 / request_tokens.len() as f64;
                score += intent * 60.0;
                if hits > 0 {
                    reasons.push(format!("intent match {}/{}", hits, request_tokens.len()));
                }
            }

            // Task-type affinity.
            if let Some(task_type) = &req.task_type {
                if haystack.contains(&task_type.to_lowercase()) {
                    score += 10.0;
                    reasons.push("task-type affinity".to_string());
                }
            }

            // Source trust contributes, but never overrides eligibility.
            let trust = f64::from(skill.source.trust_rank()) / 100.0;
            score += trust * 15.0;
            reasons.push(format!("trust {}", skill.source));

            // Historical success is a tie-breaker only; it never grants trust.
            let metrics = registry.metrics(&skill.id);
            if metrics.total_runs > 0 {
                score += metrics.success_rate() * 10.0;
                reasons.push(format!(
                    "history {:.0}% ({} runs)",
                    metrics.success_rate() * 100.0,
                    metrics.total_runs
                ));
            }

            // Prefer narrowly-scoped skills (fewer capabilities) when intent ties.
            score += (24usize.saturating_sub(skill.capabilities_required.len()) as f64) * 0.2;

            out.push(RankedSkill {
                skill,
                score,
                reasons,
            });
        }

        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out.truncate(req.limit.max(1));
        out
    }
}
