use super::EnvOr;
use anyhow::Result;
use infra::llm::{EmbedModelConfig, ModelConfig, ModelParameters, Provider};
use secrecy::SecretString;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawLlmConfig {
    providers: HashMap<String, RawProvider>,
    interests: RawInterestsModels,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInterestsModels {
    summary_model: String,
    embed_model: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProvider {
    base_url: EnvOr<String>,
    key: Option<EnvOr<SecretString>>,
    #[serde(default)]
    models: HashMap<String, RawModel>,
    #[serde(default)]
    embed_models: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawModel {
    #[serde(default)]
    max_tokens: Option<EnvOr<u32>>,
    #[serde(default)]
    temperature: Option<EnvOr<f32>>,
}

impl RawLlmConfig {
    pub(super) fn into_config(self) -> Result<LlmConfig> {
        let mut providers = HashMap::new();
        let mut models = HashMap::new();
        let mut embed_models = HashMap::new();

        for (provider_name, provider) in self.providers {
            let api_key = provider.key.map(|k| k.resolve_secret()).transpose()?;

            providers.insert(
                provider_name.clone(),
                Provider {
                    base_url: provider.base_url.resolve()?,
                    api_key,
                },
            );

            for (model_name, model) in provider.models {
                models.insert(
                    format!("{provider_name}:{model_name}"),
                    ModelConfig {
                        provider_name: provider_name.clone(),
                        params: ModelParameters {
                            max_tokens: model.max_tokens.map(|v| v.resolve()).transpose()?,
                            temperature: model.temperature.map(|v| v.resolve()).transpose()?,
                            ..Default::default()
                        },
                    },
                );
            }

            for model_name in provider.embed_models {
                embed_models.insert(
                    format!("{provider_name}:{model_name}"),
                    EmbedModelConfig {
                        provider_name: provider_name.clone(),
                    },
                );
            }
        }

        Ok(LlmConfig {
            providers,
            models,
            embed_models,
            interests_summary_model: self.interests.summary_model,
            interests_embed_model: self.interests.embed_model,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub providers: HashMap<String, Provider>,
    pub models: HashMap<String, ModelConfig>,
    pub embed_models: HashMap<String, EmbedModelConfig>,
    pub interests_summary_model: String,
    pub interests_embed_model: String,
}
