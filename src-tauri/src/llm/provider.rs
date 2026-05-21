use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role { System, User, Assistant, Tool }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,  // JSON-encoded string per OpenAI convention
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON schema
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseFormat {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json_object")]
    JsonObject,
    #[serde(rename = "json_schema")]
    JsonSchema { name: String, schema: serde_json::Value, strict: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
    pub finish_reason: String,
    pub model: String,
}

#[derive(Debug, Clone)]
pub enum StreamChunk {
    Text(String),
    ToolCallDelta { id: String, index: u32, name: Option<String>, arguments_delta: String },
    Done { usage: Option<Usage>, finish_reason: String },
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn default_model(&self) -> &'static str;
    fn default_embedding_model(&self) -> &'static str;

    async fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse>;
    async fn stream(&self, req: CompletionRequest) -> AppResult<BoxStream<'static, AppResult<StreamChunk>>>;
    async fn embed(&self, model: Option<String>, inputs: Vec<String>) -> AppResult<Vec<Vec<f32>>>;

    async fn translate(&self, text: &str, target_lang: &str) -> AppResult<String> {
        let req = CompletionRequest {
            model: self.default_model().to_string(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: Some(format!(
                        "You are a translator. Translate the user's text to {target_lang}. \
                         Preserve formatting, code, and proper nouns. \
                         Return ONLY the translation, no commentary, no quotes."
                    )),
                    tool_calls: vec![],
                    tool_call_id: None,
                    name: None,
                },
                Message {
                    role: Role::User,
                    content: Some(text.to_string()),
                    tool_calls: vec![],
                    tool_call_id: None,
                    name: None,
                },
            ],
            tools: vec![],
            temperature: Some(0.2),
            max_tokens: None,
            response_format: None,
        };
        let resp = self.complete(req).await?;
        Ok(resp.content.unwrap_or_default())
    }
}
