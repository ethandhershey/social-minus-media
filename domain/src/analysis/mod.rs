pub mod grading;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProductGrade {
    pub score: u8,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementSummary {
    pub suggestions: Vec<String>,
}

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// pub enum LlmProvider {
//     OpenAi,
//     Anthropic,
//     Google,
// }
