use crate::network::NetworkSecurityService;
use crate::privacy::{DecisionState, PrivacyClassification, PrivacyGate};
use crate::tools::errors::ToolError;
use crate::tools::request::ToolRequest;
use crate::tools::response::ToolDecision;
use crate::tools::types::{RiskLevel, ToolCapability, ToolDescriptor, ToolSource};

pub struct ToolPolicyEvaluator;

impl ToolPolicyEvaluator {
    pub fn evaluate(
        request: &ToolRequest,
        descriptor: &ToolDescriptor,
        privacy_gate: &PrivacyGate,
        network_security: &NetworkSecurityService,
    ) -> Result<ToolDecision, ToolError> {
        // INVARIANT 2: Unknown / disabled tool => BLOCK
        if !descriptor.enabled {
            return Ok(ToolDecision::block(
                format!("Tool '{}' is disabled by policy", descriptor.id),
                "system_tool_policy",
            ));
        }

        // INVARIANT 3: Validate requested capabilities against declared tool capabilities
        // If request asks for a capability the tool descriptor never declared => BLOCK (Escalation prevention)
        for req_cap in &request.requested_capabilities {
            if !descriptor.capabilities.contains(req_cap) {
                return Ok(ToolDecision::block(
                    format!(
                        "Tool '{}' attempted capability escalation: requested '{}' which is not declared",
                        descriptor.id, req_cap
                    ),
                    "capability_escalation_defense",
                ));
            }
        }

        // INVARIANT 15: MCP Servers are untrusted by default
        if descriptor.source == ToolSource::MCP {
            // Check MCP trust & policy restrictions
            if descriptor
                .capabilities
                .contains(&ToolCapability::ProcessExecute)
                || descriptor
                    .capabilities
                    .contains(&ToolCapability::ProcessSpawn)
            {
                return Ok(ToolDecision::block(
                    format!(
                        "MCP server tool '{}' cannot request process execution capability",
                        descriptor.id
                    ),
                    "mcp_security_boundary",
                ));
            }
        }

        // Evaluate Privacy Gate (Phase 6 Integration)
        if let Some(ref priv_ctx) = request.privacy_context {
            if priv_ctx.workspace_classification == PrivacyClassification::Secret
                && !request
                    .requested_capabilities
                    .contains(&ToolCapability::ReadSecretData)
            {
                return Ok(ToolDecision::block(
                    "Tool requested access to SECRET classified data without ReadSecretData capability",
                    "privacy_policy_gate",
                ));
            }

            // Check if privacy gate consent is required or denied
            let priv_dec = privacy_gate.evaluate(priv_ctx);
            if priv_dec.decision == DecisionState::DenyOnline {
                return Ok(ToolDecision::block(
                    format!("Privacy Gate denied tool access: {}", priv_dec.reason),
                    "privacy_policy_gate",
                ));
            }
        }

        // Evaluate Network Security (Phase 7 Integration)
        if request
            .requested_capabilities
            .contains(&ToolCapability::ExternalHttps)
            || request
                .requested_capabilities
                .contains(&ToolCapability::ExternalHttp)
            || request
                .requested_capabilities
                .contains(&ToolCapability::WebFetch)
            || request
                .requested_capabilities
                .contains(&ToolCapability::WebSearch)
        {
            if let Some(target_url) = request.arguments.get("url").and_then(|u| u.as_str()) {
                let endpoint = match crate::network::NetworkEndpoint::parse(target_url) {
                    Ok(ep) => ep,
                    Err(e) => {
                        return Ok(ToolDecision::block(
                            format!("Invalid network target URL '{}': {}", target_url, e),
                            "network_security_boundary",
                        ));
                    }
                };

                let mut net_req = crate::network::NetworkRequest::new(
                    "tool_runtime",
                    crate::network::NetworkCapability::WebFetch,
                    endpoint,
                );
                net_req.task_id = request.task_id.clone();
                net_req.session_id = request.session_id.clone();
                net_req.workspace_id = request.workspace_id.clone();

                let net_dec = network_security.evaluate_request(&net_req);
                if !net_dec.is_allowed() {
                    return Ok(ToolDecision::block(
                        format!(
                            "Network Security boundary denied tool egress to '{}': {}",
                            target_url, net_dec.reason
                        ),
                        "network_security_boundary",
                    ));
                }
            }
        }

        // Determine consent requirement
        let requires_user_consent = descriptor.requires_confirmation
            || descriptor.risk_level >= RiskLevel::High
            || request
                .requested_capabilities
                .contains(&ToolCapability::ProcessExecute)
            || request
                .requested_capabilities
                .contains(&ToolCapability::FilesystemDelete);

        if requires_user_consent {
            return Ok(ToolDecision::require_consent(
                format!(
                    "Tool '{}' requires explicit user approval before execution",
                    descriptor.id
                ),
                request.requested_capabilities.clone(),
                descriptor.risk_level,
            ));
        }

        // Default: ALLOW with declared capabilities granted
        Ok(ToolDecision::allow(
            format!("Tool '{}' authorized by system policy", descriptor.id),
            descriptor.capabilities.clone(),
            descriptor.risk_level,
        ))
    }
}
