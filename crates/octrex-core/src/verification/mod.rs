pub mod checks;
pub mod errors;
pub mod evidence;
pub mod gate;
pub mod policy;
pub mod repository;
pub mod service;
pub mod types;

#[cfg(test)]
pub mod tests;

pub use checks::{
    CommandEvidenceInput, CommandKind, ExpectedArtifact, ExpectedFile, StepEvidenceInput,
    TaskConstraint, TaskVerificationRequest, ToolEvidenceInput, UserRequirement,
};
pub use errors::VerificationError;
pub use evidence::{VerificationEvidence, MAX_COMMAND_OUTPUT_CHARS, MAX_FILE_PREVIEW_CHARS};
pub use gate::{CompletionGate, GateInput};
pub use policy::VerificationPolicy;
pub use repository::{SqliteVerificationRepository, VerificationRunRecord};
pub use service::{RepairPolicy, VerificationEngine};
pub use types::{
    CheckSeverity, CheckStatus, CompletionDecision, VerificationAction, VerificationCheck,
    VerificationCheckType, VerificationResult, VerificationStatus,
};
