use std::collections::HashMap;
use std::sync::RwLock;

use crate::local_runtime::errors::LocalRuntimeError;
use crate::local_runtime::types::{
    now_timestamp, LocalModelRecord, LocalModelState, LocalRuntimeDescriptor, LocalRuntimeHealth,
};

/// Allowed lifecycle transitions. Anything not listed is denied fail-closed,
/// so the store can never claim `Running`/`Loaded` without an explicit,
/// runtime-confirmed transition.
fn allowed_transitions() -> HashMap<LocalModelState, Vec<LocalModelState>> {
    use LocalModelState::*;
    let mut m = HashMap::new();
    m.insert(
        Discovered,
        vec![Registered, Available, Unavailable, Failed, Disabled],
    );
    m.insert(Registered, vec![Available, Unavailable, Disabled, Failed]);
    m.insert(Available, vec![Loading, Disabled, Unavailable, Failed]);
    m.insert(Loading, vec![Loaded, Failed, Unavailable]);
    m.insert(Loaded, vec![Running, Unloading, Unavailable, Failed]);
    m.insert(Running, vec![Loaded, Unloading, Unavailable, Failed]);
    m.insert(Unloading, vec![Available, Unavailable, Failed]);
    m.insert(Unavailable, vec![Available, Loading, Disabled, Failed]);
    m.insert(Failed, vec![Available, Disabled, Unavailable]);
    m.insert(Disabled, vec![Registered, Available]);
    m
}

// ============================================================================
// LOCAL RUNTIME REGISTRY
// ============================================================================

pub struct LocalRuntimeRegistry {
    runtimes: RwLock<HashMap<String, LocalRuntimeDescriptor>>,
}

impl Default for LocalRuntimeRegistry {
    fn default() -> Self {
        Self {
            runtimes: RwLock::new(HashMap::new()),
        }
    }
}

impl LocalRuntimeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, descriptor: LocalRuntimeDescriptor) -> Result<(), LocalRuntimeError> {
        let mut guard = self
            .runtimes
            .write()
            .map_err(|e| LocalRuntimeError::Internal {
                reason: format!("Runtime registry lock poisoned: {}", e),
            })?;
        if guard.contains_key(&descriptor.id) {
            return Err(LocalRuntimeError::DuplicateRuntime {
                runtime_id: descriptor.id.clone(),
            });
        }
        guard.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn upsert(&self, descriptor: LocalRuntimeDescriptor) -> Result<(), LocalRuntimeError> {
        let mut guard = self
            .runtimes
            .write()
            .map_err(|e| LocalRuntimeError::Internal {
                reason: format!("Runtime registry lock poisoned: {}", e),
            })?;
        guard.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn get(&self, runtime_id: &str) -> Option<LocalRuntimeDescriptor> {
        self.runtimes.read().ok()?.get(runtime_id).cloned()
    }

    pub fn list(&self) -> Vec<LocalRuntimeDescriptor> {
        self.runtimes
            .read()
            .map(|g| g.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn remove(&self, runtime_id: &str) -> Option<LocalRuntimeDescriptor> {
        self.runtimes.write().ok()?.remove(runtime_id)
    }

    pub fn update_health(
        &self,
        runtime_id: &str,
        health: LocalRuntimeHealth,
        error: Option<String>,
    ) -> Result<(), LocalRuntimeError> {
        let mut guard = self
            .runtimes
            .write()
            .map_err(|e| LocalRuntimeError::Internal {
                reason: format!("Runtime registry lock poisoned: {}", e),
            })?;
        let entry =
            guard
                .get_mut(runtime_id)
                .ok_or_else(|| LocalRuntimeError::RuntimeNotFound {
                    runtime_id: runtime_id.to_string(),
                })?;
        entry.health = health;
        entry.last_checked_timestamp = Some(now_timestamp());
        entry.last_error = error;
        Ok(())
    }

    pub fn routable_runtimes(&self) -> Vec<LocalRuntimeDescriptor> {
        self.list()
            .into_iter()
            .filter(|r| r.health.is_routable())
            .collect()
    }
}

// ============================================================================
// LOCAL MODEL STORE (lifecycle authority, syncs descriptors to ModelRegistry)
// ============================================================================

pub struct LocalModelStore {
    models: RwLock<HashMap<String, LocalModelRecord>>,
}

impl Default for LocalModelStore {
    fn default() -> Self {
        Self {
            models: RwLock::new(HashMap::new()),
        }
    }
}

impl LocalModelStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, record: LocalModelRecord) -> Result<(), LocalRuntimeError> {
        let mut guard = self
            .models
            .write()
            .map_err(|e| LocalRuntimeError::Internal {
                reason: format!("Model store lock poisoned: {}", e),
            })?;
        if guard.contains_key(&record.id) {
            return Err(LocalRuntimeError::DuplicateModel {
                model_id: record.id.clone(),
            });
        }
        guard.insert(record.id.clone(), record);
        Ok(())
    }

    pub fn upsert(&self, record: LocalModelRecord) -> Result<(), LocalRuntimeError> {
        let mut guard = self
            .models
            .write()
            .map_err(|e| LocalRuntimeError::Internal {
                reason: format!("Model store lock poisoned: {}", e),
            })?;
        guard.insert(record.id.clone(), record);
        Ok(())
    }

    pub fn get(&self, model_id: &str) -> Option<LocalModelRecord> {
        self.models.read().ok()?.get(model_id).cloned()
    }

    pub fn get_by_registry_id(&self, registry_model_id: &str) -> Option<LocalModelRecord> {
        self.models
            .read()
            .ok()?
            .values()
            .find(|m| m.registry_model_id == registry_model_id)
            .cloned()
    }

    pub fn list(&self) -> Vec<LocalModelRecord> {
        self.models
            .read()
            .map(|g| g.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn list_by_runtime(&self, runtime_id: &str) -> Vec<LocalModelRecord> {
        self.list()
            .into_iter()
            .filter(|m| m.runtime_id == runtime_id)
            .collect()
    }

    pub fn routable_models(&self) -> Vec<LocalModelRecord> {
        self.list()
            .into_iter()
            .filter(|m| m.state.is_routable() && m.health.is_routable())
            .collect()
    }

    /// Explicit lifecycle transition with a fixed transition table.
    pub fn transition(
        &self,
        model_id: &str,
        next: LocalModelState,
        error: Option<String>,
    ) -> Result<LocalModelRecord, LocalRuntimeError> {
        let allowed = allowed_transitions();
        let mut guard = self
            .models
            .write()
            .map_err(|e| LocalRuntimeError::Internal {
                reason: format!("Model store lock poisoned: {}", e),
            })?;
        let entry = guard
            .get_mut(model_id)
            .ok_or_else(|| LocalRuntimeError::ModelNotFound {
                model_id: model_id.to_string(),
            })?;
        let valid = allowed
            .get(&entry.state)
            .map(|v| v.contains(&next))
            .unwrap_or(false);
        if !valid {
            return Err(LocalRuntimeError::LifecycleDenied {
                reason: format!(
                    "Transition {} -> {} is not permitted for model '{}'",
                    entry.state, next, model_id
                ),
            });
        }
        entry.state = next;
        entry.updated_at_timestamp = now_timestamp();
        if next == LocalModelState::Failed || next == LocalModelState::Unavailable {
            entry.health = LocalRuntimeHealth::Unavailable;
            entry.last_error = error;
        } else if next == LocalModelState::Disabled {
            entry.last_error = None;
        }
        Ok(entry.clone())
    }

    pub fn set_health(
        &self,
        model_id: &str,
        health: LocalRuntimeHealth,
        error: Option<String>,
    ) -> Result<(), LocalRuntimeError> {
        let mut guard = self
            .models
            .write()
            .map_err(|e| LocalRuntimeError::Internal {
                reason: format!("Model store lock poisoned: {}", e),
            })?;
        let entry = guard
            .get_mut(model_id)
            .ok_or_else(|| LocalRuntimeError::ModelNotFound {
                model_id: model_id.to_string(),
            })?;
        entry.health = health;
        entry.last_error = error;
        entry.updated_at_timestamp = now_timestamp();
        Ok(())
    }

    pub fn remove(&self, model_id: &str) -> Option<LocalModelRecord> {
        self.models.write().ok()?.remove(model_id)
    }
}

// ============================================================================
// BOUNDED EXECUTION SLOTS (constrained-hardware queueing)
// ============================================================================

/// Bounded execution tracker: at most `max_concurrent` models may hold a
/// running slot. Excess requests are rejected with guidance — never silently
/// queued behind user-critical work, and never resolved by killing processes
/// Octrex does not own.
pub struct ExecutionSlots {
    max_concurrent: usize,
    active: RwLock<HashMap<String, String>>,
}

impl ExecutionSlots {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent: max_concurrent.max(1),
            active: RwLock::new(HashMap::new()),
        }
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
    }

    /// Try to acquire a slot for `model_id`. Returns `Ok(())` when the model
    /// already holds a slot or a free slot exists; `Err` names the occupant.
    pub fn try_acquire(&self, model_id: &str, call_id: &str) -> Result<(), String> {
        let mut guard = self
            .active
            .write()
            .map_err(|e| format!("Slot lock poisoned: {}", e))?;
        if guard.contains_key(model_id) {
            guard.insert(model_id.to_string(), call_id.to_string());
            return Ok(());
        }
        if guard.len() < self.max_concurrent {
            guard.insert(model_id.to_string(), call_id.to_string());
            return Ok(());
        }
        let occupants: Vec<String> = guard.keys().cloned().collect();
        Err(format!(
            "Resource limit reached: model '{}' cannot start while [{}] hold the {} execution slot(s). Wait, cancel, or explicitly unload the occupant first.",
            model_id,
            occupants.join(", "),
            self.max_concurrent
        ))
    }

    pub fn release_model(&self, model_id: &str) {
        if let Ok(mut guard) = self.active.write() {
            guard.remove(model_id);
        }
    }

    pub fn release_call(&self, call_id: &str) {
        if let Ok(mut guard) = self.active.write() {
            guard.retain(|_, v| v != call_id);
        }
    }

    pub fn occupants(&self) -> Vec<String> {
        self.active
            .read()
            .map(|g| g.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn is_active(&self, model_id: &str) -> bool {
        self.active
            .read()
            .map(|g| g.contains_key(model_id))
            .unwrap_or(false)
    }
}
