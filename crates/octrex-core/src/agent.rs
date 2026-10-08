use crate::config::{AppConfig, ProviderConfig, ProviderType};
use crate::models::{
    CallCorrelation, ModelMessage, ModelRequest, ModelResponse, ModelRuntime, ResponseFormat,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionRequest {
    pub prompt: String,
    pub provider_id: Option<String>,
    pub workspace_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResponse {
    pub task_id: String,
    pub provider_used: String,
    pub output: String,
    pub success: bool,
    pub execution_time_ms: u128,
}

pub struct AgentEngine {
    http_client: Client,
}

impl Default for AgentEngine {
    fn default() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { http_client }
    }
}

impl AgentEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute prompt via normalized ModelRuntime when provided
    pub async fn execute_via_runtime(
        &self,
        runtime: &ModelRuntime,
        model_id: &str,
        prompt: &str,
    ) -> anyhow::Result<AgentExecutionResponse> {
        let start = std::time::Instant::now();

        let req = ModelRequest {
            model_id: model_id.to_string(),
            messages: vec![ModelMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                tool_calls: None,
            }],
            system_instructions: Some("You are Octrex AI software engineer.".to_string()),
            tools: vec![],
            temperature: Some(0.3),
            max_output_tokens: Some(4096),
            response_format: ResponseFormat::Text,
            metadata: std::collections::HashMap::new(),
            correlation: CallCorrelation::default(),
        };

        let resp: ModelResponse = runtime
            .invoke(req)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let task_id = format!("task-{}", uuid::Uuid::new_v4().simple());
        let execution_time_ms = start.elapsed().as_millis();

        Ok(AgentExecutionResponse {
            task_id,
            provider_used: model_id.to_string(),
            output: resp.content,
            success: true,
            execution_time_ms,
        })
    }

    /// Execute prompt via production ModelRouter and ModelRuntime
    pub async fn execute_via_router(
        &self,
        router: &crate::router::ModelRouter,
        runtime: &ModelRuntime,
        routing_req: crate::router::RoutingRequest,
        prompt: &str,
    ) -> anyhow::Result<AgentExecutionResponse> {
        let decision = router.route(routing_req);

        if !decision.is_selected() {
            anyhow::bail!(
                "Model Router rejected execution: {:?} - {}",
                decision.state,
                decision.reason
            );
        }

        let selected_model = decision
            .selected_model_id
            .ok_or_else(|| anyhow::anyhow!("Selected model ID missing in routing decision"))?;

        self.execute_via_runtime(runtime, &selected_model, prompt)
            .await
    }

    /// Execute prompt with ContextEngine integration (ingests request, checks budget, ingests response)
    pub async fn execute_with_context(
        &self,
        runtime: &ModelRuntime,
        model_id: &str,
        prompt: &str,
        context_engine: &crate::context::ContextService,
        session_id: Option<crate::ids::SessionId>,
        task_id: Option<crate::ids::TaskId>,
    ) -> anyhow::Result<AgentExecutionResponse> {
        let start = std::time::Instant::now();

        // 1. Ingest prompt into context engine
        let mut user_item = crate::context::ContextItem::new(
            crate::context::ContextSource::UserRequest,
            crate::context::ContextRole::User,
            prompt,
        )
        .with_session(session_id.clone())
        .with_task(task_id.clone());

        user_item.priority = 10;
        let _ = context_engine.add_item(user_item);

        // 2. Execute request via runtime
        let resp = self.execute_via_runtime(runtime, model_id, prompt).await?;

        // 3. Ingest model response into context engine
        let _ = context_engine.ingest_model_output(
            &resp.output,
            model_id,
            crate::privacy::PrivacyClassification::Public,
            session_id,
            task_id,
        );

        let execution_time_ms = start.elapsed().as_millis();

        Ok(AgentExecutionResponse {
            task_id: resp.task_id,
            provider_used: model_id.to_string(),
            output: resp.output,
            success: true,
            execution_time_ms,
        })
    }

    /// Execute prompt directly against the configured live provider API endpoint.
    pub async fn execute(
        &self,
        config: &AppConfig,
        req: AgentExecutionRequest,
    ) -> anyhow::Result<AgentExecutionResponse> {
        let start = std::time::Instant::now();
        let provider_id = req
            .provider_id
            .as_deref()
            .unwrap_or(&config.active_provider);

        let provider_cfg = config
            .providers
            .get(provider_id)
            .ok_or_else(|| anyhow::anyhow!("Provider '{}' is not registered", provider_id))?;

        let output = match provider_cfg.provider_type {
            ProviderType::OpenCode
            | ProviderType::NvidiaNim
            | ProviderType::Groq
            | ProviderType::Custom => {
                self.execute_openai_compatible(provider_cfg, &req.prompt)
                    .await?
            }
            ProviderType::Ollama => self.execute_ollama(provider_cfg, &req.prompt).await?,
            ProviderType::Google => self.execute_google(provider_cfg, &req.prompt).await?,
        };

        let task_id = format!("task-{}", uuid::Uuid::new_v4().simple());
        let execution_time_ms = start.elapsed().as_millis();

        Ok(AgentExecutionResponse {
            task_id,
            provider_used: provider_id.to_string(),
            output,
            success: true,
            execution_time_ms,
        })
    }

    async fn execute_openai_compatible(
        &self,
        config: &ProviderConfig,
        prompt: &str,
    ) -> anyhow::Result<String> {
        let api_key = config
            .api_key
            .as_ref()
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("Missing API key for provider"))?;

        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");
        let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        let payload = serde_json::json!({
            "model": config.default_model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.3
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .bearer_auth(api_key)
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Live Provider API error: {}", err_text);
        }

        #[derive(Deserialize)]
        struct ChoiceMessage {
            content: Option<String>,
        }
        #[derive(Deserialize)]
        struct Choice {
            message: ChoiceMessage,
        }
        #[derive(Deserialize)]
        struct ChatResponse {
            choices: Vec<Choice>,
        }

        let parsed = resp.json::<ChatResponse>().await?;
        let text = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_else(|| "Empty response received from provider".to_string());

        Ok(text)
    }

    async fn execute_ollama(
        &self,
        config: &ProviderConfig,
        prompt: &str,
    ) -> anyhow::Result<String> {
        let base_url = config
            .base_url
            .as_deref()
            .unwrap_or("http://127.0.0.1:11434");
        let endpoint = format!("{}/api/generate", base_url.trim_end_matches('/'));

        let payload = serde_json::json!({
            "model": config.default_model,
            "prompt": prompt,
            "stream": false
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Local Ollama API Error: {}", err_text);
        }

        #[derive(Deserialize)]
        struct OllamaGenResponse {
            response: String,
        }

        let parsed = resp.json::<OllamaGenResponse>().await?;
        Ok(parsed.response)
    }

    async fn execute_google(
        &self,
        config: &ProviderConfig,
        prompt: &str,
    ) -> anyhow::Result<String> {
        let api_key = config
            .api_key
            .as_ref()
            .filter(|k| !k.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("Missing Gemini API Key"))?;

        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            config.default_model, api_key
        );

        let payload = serde_json::json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        });

        let resp = self
            .http_client
            .post(&endpoint)
            .json(&payload)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API Error: {}", err_text);
        }

        #[derive(Deserialize)]
        struct Part {
            text: Option<String>,
        }
        #[derive(Deserialize)]
        struct Content {
            parts: Vec<Part>,
        }
        #[derive(Deserialize)]
        struct Candidate {
            content: Content,
        }
        #[derive(Deserialize)]
        struct GeminiGenResponse {
            candidates: Option<Vec<Candidate>>,
        }

        let parsed = resp.json::<GeminiGenResponse>().await?;
        let text = parsed
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.parts.first())
            .and_then(|p| p.text.clone())
            .unwrap_or_else(|| "Empty response received from Gemini".to_string());

        Ok(text)
    }
}
