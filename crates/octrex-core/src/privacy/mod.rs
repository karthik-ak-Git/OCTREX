pub mod classifier;
pub mod consent;
pub mod decision;
pub mod evidence;
pub mod gate;
pub mod policy;
pub mod repository;
pub mod rules;
#[cfg(test)]
mod tests;
pub mod types;

pub use classifier::PrivacyClassifier;
pub use consent::ConsentManager;
pub use decision::DecisionFormatter;
pub use evidence::EvidenceManager;
pub use gate::PrivacyGate;
pub use policy::PolicyEngine;
pub use repository::{PrivacyRepository, SqlitePrivacyRepository};
pub use rules::get_default_policy_rules;
pub use types::*;
