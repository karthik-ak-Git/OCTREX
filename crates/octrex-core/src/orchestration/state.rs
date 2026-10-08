use super::errors::OrchestrationError;
use super::types::OrchestratorState;

pub struct OrchestratorStateMachine {
    current_state: OrchestratorState,
}

impl OrchestratorStateMachine {
    pub fn new() -> Self {
        Self {
            current_state: OrchestratorState::Created,
        }
    }

    pub fn with_state(state: OrchestratorState) -> Self {
        Self {
            current_state: state,
        }
    }

    pub fn state(&self) -> OrchestratorState {
        self.current_state
    }

    pub fn can_transition_to(&self, target: OrchestratorState) -> bool {
        if self.current_state == target {
            return true;
        }

        match self.current_state {
            OrchestratorState::Created => matches!(
                target,
                OrchestratorState::Planning
                    | OrchestratorState::Executing
                    | OrchestratorState::Cancelled
                    | OrchestratorState::Failed
            ),
            OrchestratorState::Planning => matches!(
                target,
                OrchestratorState::PlanReady
                    | OrchestratorState::Executing
                    | OrchestratorState::Failed
                    | OrchestratorState::Cancelled
            ),
            OrchestratorState::PlanReady => matches!(
                target,
                OrchestratorState::Executing
                    | OrchestratorState::Cancelled
                    | OrchestratorState::Failed
            ),
            OrchestratorState::Executing => matches!(
                target,
                OrchestratorState::WaitingForTool
                    | OrchestratorState::WaitingForUser
                    | OrchestratorState::Verifying
                    | OrchestratorState::Retrying
                    | OrchestratorState::Blocked
                    | OrchestratorState::Completed
                    | OrchestratorState::Failed
                    | OrchestratorState::Cancelled
            ),
            OrchestratorState::WaitingForTool => matches!(
                target,
                OrchestratorState::Executing
                    | OrchestratorState::Blocked
                    | OrchestratorState::Failed
                    | OrchestratorState::Cancelled
            ),
            OrchestratorState::WaitingForUser => matches!(
                target,
                OrchestratorState::Executing
                    | OrchestratorState::Cancelled
                    | OrchestratorState::Failed
            ),
            OrchestratorState::Verifying => matches!(
                target,
                OrchestratorState::Executing
                    | OrchestratorState::Completed
                    | OrchestratorState::Failed
                    | OrchestratorState::Cancelled
            ),
            OrchestratorState::Retrying => matches!(
                target,
                OrchestratorState::Executing
                    | OrchestratorState::Blocked
                    | OrchestratorState::Failed
                    | OrchestratorState::Cancelled
            ),
            OrchestratorState::Blocked => matches!(
                target,
                OrchestratorState::Planning
                    | OrchestratorState::Executing
                    | OrchestratorState::WaitingForUser
                    | OrchestratorState::Failed
                    | OrchestratorState::Cancelled
            ),
            // Terminal states cannot transition to non-terminal states
            OrchestratorState::Completed
            | OrchestratorState::Failed
            | OrchestratorState::Cancelled => false,
        }
    }

    pub fn transition_to(&mut self, target: OrchestratorState) -> Result<(), OrchestrationError> {
        if !self.can_transition_to(target) {
            return Err(OrchestrationError::InvalidStateTransition {
                from: self.current_state.to_string(),
                to: target.to_string(),
            });
        }
        self.current_state = target;
        Ok(())
    }
}
