use crate::hardware::{
    CompatibilityResult, CompatibilityStatus, HardwareProfile, HardwareService,
    ModelCompatibilityEngine,
};
use crate::models::types::ModelDescriptor;
use crate::providers::ExecutionMode;
use std::sync::Arc;

pub struct HardwareEvaluator {
    hardware_service: Arc<HardwareService>,
}

impl HardwareEvaluator {
    pub fn new(hardware_service: Arc<HardwareService>) -> Self {
        Self { hardware_service }
    }

    pub fn evaluate_candidate(
        &self,
        model: &ModelDescriptor,
    ) -> (bool, CompatibilityResult, String) {
        // Cloud models skip local hardware bounds
        if model.execution_mode == ExecutionMode::Cloud {
            let profile = self.hardware_service.get_profile();
            let res = ModelCompatibilityEngine::evaluate(&profile, model);
            return (
                true,
                res,
                "Cloud execution target (no local hardware constraints)".to_string(),
            );
        }

        let profile: HardwareProfile = self.hardware_service.get_profile();
        let res = ModelCompatibilityEngine::evaluate(&profile, model);

        match res.status {
            CompatibilityStatus::Compatible => (
                true,
                res,
                "Hardware fully compatible with model requirements".to_string(),
            ),
            CompatibilityStatus::CompatibleWithWarnings => (
                true,
                res,
                "Hardware compatible with operational warnings".to_string(),
            ),
            CompatibilityStatus::Incompatible => (
                false,
                res,
                "Hardware is incompatible with model requirements".to_string(),
            ),
            CompatibilityStatus::Unknown => (
                false,
                res,
                "Hardware compatibility status is unknown (failing closed)".to_string(),
            ),
        }
    }
}
