use std::collections::HashMap;

use anyhow::Context as _;
use domain::error::DomainError;
use domain::ports::LlmService;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest, ChatResponseFormat, JsonSpec};
use genai::resolver::{AuthData, AuthResolver, Endpoint, ServiceTargetResolver};
use genai::{ClientConfig, ServiceTarget};
use secrecy::{ExposeSecret, SecretString};

use crate::http::HttpClient;

#[derive(Debug, Clone)]
pub struct LlmSettings {
    pub models: HashMap<String, ModelSettings>,
}

#[derive(Debug, Clone)]
pub struct ModelSettings {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<SecretString>,
    pub override_format_openai: bool,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

pub struct GenAiService {
    client: genai::Client,
    settings: LlmSettings,
}

impl GenAiService {
    pub fn new(http_client: HttpClient, settings: LlmSettings) -> Self {
        let cloned_settings = settings.clone();

        let target_resolver = ServiceTargetResolver::from_resolver_fn(
            move |mut service_target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
                let model_name = service_target.model.model_name.to_string();

                let model_settings = settings.models.get(&model_name)
                    .ok_or_else(|| genai::resolver::Error::Custom(format!(
                        "unknown model: {model_name}"
                    )))?;

                if model_settings.override_format_openai {
                    service_target.model.adapter_kind = genai::adapter::AdapterKind::OpenAI;
                }

                if let Some(key) = &model_settings.api_key {
                    service_target.auth = AuthData::from_single(key.expose_secret());
                }

                service_target.endpoint = Endpoint::from_owned(model_settings.base_url.clone());

                Ok(service_target)
            },
        );

        Self {
            client: genai::Client::builder()
                .with_reqwest(http_client.into())
                .with_config(
                    ClientConfig::default()
                        .with_auth_resolver(AuthResolver::from_resolver_fn(|_| {
                            Ok(Some(AuthData::from_single(String::new())))
                        }))
                        .with_service_target_resolver(target_resolver),
                )
                .build(),
            settings: cloned_settings,
        }
    }
}

impl LlmService for GenAiService {
    fn available_models(&self) -> Vec<String> {
        self.settings.models.keys().cloned().collect()
    }

    async fn get_simple_response(
        &self,
        model: &str,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, DomainError> {
        let chat_req = ChatRequest::from_messages(vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_message),
        ]);

        let response = self
            .client
            .exec_chat(model, chat_req, None)
            .await
            .map_err(|_| DomainError::AiServiceUnavailable)?;

        Ok(response
            .content
            .first_text()
            .context("llm didn't return any text")
            .map_err(|_| DomainError::AiServiceUnavailable)?
            .to_string())
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
        let schema_value = schemars::schema_for!(T).to_value();
        let spec = JsonSpec::new("response", schema_value);
        let chat_options =
            ChatOptions::default().with_response_format(ChatResponseFormat::from(spec));

        let chat_req = ChatRequest::from_messages(vec![
            ChatMessage::system(system_prompt),
            ChatMessage::user(user_message),
        ]);

        let response = self
            .client
            .exec_chat(model, chat_req, Some(&chat_options))
            .await
            .map_err(|_| DomainError::AiServiceUnavailable)?;

        let text = response
            .content
            .first_text()
            .context("llm didn't return any text")
            .map_err(|_| DomainError::AiServiceUnavailable)?;

        serde_json::from_str(text).map_err(|_| DomainError::AiServiceUnavailable)
    }
}
