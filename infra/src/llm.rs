use std::collections::HashMap;

use anyhow::anyhow;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use serde_json::Value;

use domain::{error::DomainError, ports::LlmService};

use crate::http::HttpClient;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Provider {
    pub base_url: String,
    pub api_key: Option<SecretString>,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub provider_name: String,
    pub params: ModelParameters,
}

#[derive(Debug, Clone)]
pub struct EmbedModelConfig {
    pub provider_name: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ModelParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

// ---------------------------------------------------------------------------
// Wire types  (OpenAI /v1/chat/completions schema)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct JsonSchemaWrapper {
    name: String,
    strict: bool,
    schema: Value,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    #[serde(flatten)]
    params: ModelParameters,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
    json_schema: JsonSchemaWrapper,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct SimpleLlmClient {
    client: Client,
    providers: HashMap<String, Provider>,
    models: HashMap<String, ModelConfig>,
    embed_models: HashMap<String, EmbedModelConfig>,
}

impl SimpleLlmClient {
    pub fn new(
        http_client: HttpClient,
        providers: HashMap<String, Provider>,
        models: HashMap<String, ModelConfig>,
        embed_models: HashMap<String, EmbedModelConfig>,
    ) -> Self {
        Self {
            client: http_client.into(),
            providers,
            models,
            embed_models,
        }
    }
}

impl SimpleLlmClient {
    fn resolve(&self, model: &str) -> Result<(&Provider, &ModelConfig), DomainError> {
        let model_config = self
            .models
            .get(model)
            .ok_or_else(|| DomainError::Internal(anyhow!("unknown model: {model}")))?;

        let provider = self
            .providers
            .get(&model_config.provider_name)
            .ok_or_else(|| {
                DomainError::Internal(anyhow!(
                    "unknown provider '{}' for model '{model}'",
                    model_config.provider_name
                ))
            })?;

        Ok((provider, model_config))
    }

    fn resolve_embed(&self, model: &str) -> Result<&Provider, DomainError> {
        let embed_config = self
            .embed_models
            .get(model)
            .ok_or_else(|| DomainError::Internal(anyhow!("unknown embed model: {model}")))?;

        let provider = self
            .providers
            .get(&embed_config.provider_name)
            .ok_or_else(|| {
                DomainError::Internal(anyhow!(
                    "unknown provider '{}' for embed model '{model}'",
                    embed_config.provider_name
                ))
            })?;

        Ok(provider)
    }

    async fn send(
        &self,
        provider: &Provider,
        body: &ChatRequest<'_>,
    ) -> Result<String, DomainError> {
        let url = format!(
            "{}/chat/completions",
            provider.base_url.trim_end_matches('/')
        );

        let mut req = self.client.post(url).json(body);

        if let Some(key) = &provider.api_key {
            req = req.bearer_auth(key.expose_secret());
        }

        let resp = req
            .send()
            .await
            .map_err(|e| DomainError::Internal(anyhow!("{e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(DomainError::Internal(anyhow!(
                "LLM API error {status}: {text}"
            )));
        }

        Ok(resp
            .json::<Value>()
            .await
            .map_err(|e| DomainError::Internal(anyhow!("failed to deserialize response: {e}")))?
            .pointer("/choices/0/message/content")
            .ok_or_else(|| DomainError::Internal(anyhow!("failed to deserialize response")))?
            .as_str()
            .ok_or_else(|| DomainError::Internal(anyhow!("empty content in LLM response")))?
            .to_string())
    }
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

impl LlmService for SimpleLlmClient {
    fn available_models(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }

    async fn get_simple_response(
        &self,
        model: &str,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, DomainError> {
        let (provider, model_config) = self.resolve(model)?;

        let body = ChatRequest {
            model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user",
                    content: user_message,
                },
            ],
            params: model_config.params.clone(),
            response_format: None,
        };

        self.send(provider, &body).await
    }

    async fn get_structured_response<T>(
        &self,
        model: &str,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<T, DomainError>
    where
        T: schemars::JsonSchema + serde::de::DeserializeOwned,
    {
        let (provider, model_config) = self.resolve(model)?;

        let schema_value = schemars::schema_for!(T).to_value();

        let schema_name = schema_value
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("response")
            .to_lowercase()
            .replace(' ', "_");

        let body = ChatRequest {
            model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user",
                    content: user_message,
                },
            ],
            params: model_config.params.clone(),
            response_format: Some(ResponseFormat {
                kind: "json_schema",
                json_schema: JsonSchemaWrapper {
                    name: schema_name,
                    strict: true,
                    schema: schema_value,
                },
            }),
        };

        let raw = self.send(provider, &body).await?;

        serde_json::from_str::<T>(&raw).map_err(|e| {
            DomainError::Internal(anyhow!(
                "structured response deserialization failed: {e}\nraw: {raw}"
            ))
        })
    }

    async fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>, DomainError> {
        let provider = self.resolve_embed(model)?;

        let url = format!("{}/embeddings", provider.base_url.trim_end_matches('/'));

        let body = EmbedRequest { model, input };
        let mut req = self.client.post(url).json(&body);

        if let Some(key) = &provider.api_key {
            req = req.bearer_auth(key.expose_secret());
        }

        let resp = req
            .send()
            .await
            .map_err(|e| DomainError::Internal(anyhow!("{e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(DomainError::Internal(anyhow!(
                "embed API error {status}: {text}"
            )));
        }

        let embedding = resp
            .json::<Value>()
            .await
            .map_err(|e| {
                DomainError::Internal(anyhow!("failed to deserialize embed response: {e}"))
            })?
            .pointer("/data/0/embedding")
            .ok_or_else(|| DomainError::Internal(anyhow!("missing embedding in response")))?
            .as_array()
            .ok_or_else(|| DomainError::Internal(anyhow!("embedding is not an array")))?
            .iter()
            .map(|v| {
                v.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| DomainError::Internal(anyhow!("non-numeric value in embedding")))
            })
            .collect::<Result<Vec<f32>, _>>()?;

        Ok(embedding)
    }
}
