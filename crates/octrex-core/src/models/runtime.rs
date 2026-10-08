use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

use crate::models::registry::ModelRegistry;
use crate::models::types::{
    ModelError, ModelMessage, ModelRequest, ModelResponse, ModelStreamEvent,
};
use crate::providers::registry::ProviderRegistry;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TokenCount {
    Exact(usize),
    Estimated(usize),
    Unknown,
}

pub trait Tokenizer: Send + Sync {
    fn count_tokens(&self, text: &str) -> TokenCount;
    fn count_messages(&self, messages: &[ModelMessage]) -> TokenCount;
}

pub struct DefaultTokenizer;

impl Tokenizer for DefaultTokenizer {
    fn count_tokens(&self, text: &str) -> TokenCount {
        if text.is_empty() {
            return TokenCount::Exact(0);
        }
        // Character count based estimation (~4 chars per token)
        let count = (text.chars().count() + 3) / 4;
        TokenCount::Estimated(count)
    }

    fn count_messages(&self, messages: &[ModelMessage]) -> TokenCount {
        let mut total = 0;
        for msg in messages {
            match self.count_tokens(&msg.content) {
                TokenCount::Estimated(n) | TokenCount::Exact(n) => total += n + 4,
                TokenCount::Unknown => return TokenCount::Unknown,
            }
        }
        TokenCount::Estimated(total)
    }
}

pub struct ModelRuntime {
    provider_registry: Arc<ProviderRegistry>,
    model_registry: Arc<ModelRegistry>,
    tokenizer: Arc<dyn Tokenizer>,
}

impl ModelRuntime {
    pub fn new(
        provider_registry: Arc<ProviderRegistry>,
        model_registry: Arc<ModelRegistry>,
    ) -> Self {
        Self {
            provider_registry,
            model_registry,
            tokenizer: Arc::new(DefaultTokenizer),
        }
    }

    pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self {
        self.tokenizer = tokenizer;
        self
    }

    pub fn tokenizer(&self) -> &dyn Tokenizer {
        self.tokenizer.as_ref()
    }

    pub async fn invoke(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        info!(
            model_id = %request.model_id,
            call_id = %request.correlation.call_id,
            "Executing normalized model request"
        );

        let descriptor = self
            .model_registry
            .get_model(&request.model_id)
            .ok_or_else(|| ModelError::ModelUnavailable {
                model_id: request.model_id.clone(),
                reason: format!(
                    "Model '{}' not registered in ModelRegistry",
                    request.model_id
                ),
            })?;

        let provider = self
            .provider_registry
            .get_provider(&descriptor.provider_id)
            .ok_or_else(|| ModelError::ProviderUnavailable {
                provider_id: descriptor.provider_id.clone(),
                reason: format!("Provider '{}' not registered", descriptor.provider_id),
            })?;

        provider.invoke(&request).await
    }

    pub async fn stream(
        &self,
        request: ModelRequest,
    ) -> Result<mpsc::Receiver<ModelStreamEvent>, ModelError> {
        info!(
            model_id = %request.model_id,
            call_id = %request.correlation.call_id,
            "Initiating normalized model streaming request"
        );

        let descriptor = self
            .model_registry
            .get_model(&request.model_id)
            .ok_or_else(|| ModelError::ModelUnavailable {
                model_id: request.model_id.clone(),
                reason: format!(
                    "Model '{}' not registered in ModelRegistry",
                    request.model_id
                ),
            })?;

        let provider = self
            .provider_registry
            .get_provider(&descriptor.provider_id)
            .ok_or_else(|| ModelError::ProviderUnavailable {
                provider_id: descriptor.provider_id.clone(),
                reason: format!("Provider '{}' not registered", descriptor.provider_id),
            })?;

        provider.stream(&request).await
    }
}
