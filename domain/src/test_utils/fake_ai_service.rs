use crate::{error::DomainError, ports::LlmService};

#[derive(Clone)]
pub struct FakeAiService {
    models: Vec<String>,
    simple_response: String,
    structured_response: serde_json::Value,
}

impl FakeAiService {
    pub fn new() -> Self {
        Self {
            models: vec!["fake".to_string()],
            simple_response: "This is a fake AI response.".to_string(),
            // Hardcoded to match ProductGrade { score: u8, summary: String }
            structured_response: serde_json::json!({
                "score": 5,
                "summary": "Fake AI summary."
            }),
        }
    }
}

impl LlmService for FakeAiService {
    fn available_models(&self) -> Vec<String> {
        self.models.clone()
    }

    async fn get_simple_response(
        &self,
        _model: &str,
        _system_prompt: &str,
        _user_message: &str,
    ) -> Result<String, DomainError> {
        Ok(self.simple_response.clone())
    }

    async fn get_structured_response<T>(
        &self,
        _model: &str,
        _system_prompt: &str,
        _user_message: &str,
    ) -> Result<T, DomainError>
    where
        T: schemars::JsonSchema + serde::de::DeserializeOwned,
    {
        serde_json::from_value(self.structured_response.clone())
            .map_err(|_| DomainError::AiServiceUnavailable)
    }

    async fn embed(&self, _model: &str, _input: &str) -> Result<Vec<f32>, DomainError> {
        Ok(vec![0.0; 1536])
    }
}
