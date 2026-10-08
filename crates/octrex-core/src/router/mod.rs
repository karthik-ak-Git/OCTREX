pub mod audit;
pub mod candidates;
pub mod constraints;
pub mod context;
pub mod decision;
pub mod errors;
pub mod events;
pub mod fallback;
pub mod hardware;
pub mod health;
pub mod policy;
pub mod request;
#[allow(clippy::module_inception)]
pub mod router;
pub mod scorer;
pub mod types;

#[cfg(test)]
pub mod tests;

pub use audit::RouterAuditLogger;
pub use candidates::CandidatePipeline;
pub use constraints::CapabilityEvaluator;
pub use context::ContextEvaluator;
pub use decision::RoutingDecisionBuilder;
pub use errors::RouterError;
pub use events::RouterEventNotifier;
pub use fallback::FallbackGuard;
pub use hardware::HardwareEvaluator;
pub use health::HealthEvaluator;
pub use policy::PolicyEvaluator;
pub use request::RoutingRequest;
pub use router::ModelRouter;
pub use scorer::ModelScorer;
pub use types::{
    CandidateEliminationReason, CandidateEvaluation, RoutingDecision, RoutingDecisionState,
    RoutingEvidence, RoutingMode,
};
