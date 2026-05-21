use futures::stream::{self, BoxStream, StreamExt};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use super::provider::{
    CompletionRequest, CompletionResponse, Message, Role, StreamChunk, ToolCall, ToolDef,
    Usage,
};

// ---------------------------------------------------------------------------
// Internal OpenAI wire types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct OaToolFunction {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Serialize)]
struct OaTool {
    r#type: &'static str,
    function: OaToolFunction,
}

#[derive(Serialize)]
struct OaMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OaToolCallOut>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize)]
struct OaToolCallOut {
    id: String,
    r#type: &'static str,
    function: OaFunctionCall,
}

#[derive(Serialize)]
struct OaFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct OaCompletionBody {
    model: String,
    messages: Vec<OaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OaTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<Value>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
}

#[derive(Serialize)]
struct StreamOptions {
    include_usage: bool,
}

// --- response deserialization ---

#[derive(Deserialize)]
struct OaCompletionResponse {
    model: String,
    choices: Vec<OaChoice>,
    usage: Option<OaUsage>,
}

#[derive(Deserialize)]
struct OaChoice {
    message: Option<OaChoiceMessage>,
    delta: Option<OaDelta>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OaChoiceMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OaToolCallIn>>,
}

#[derive(Deserialize)]
struct OaDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OaToolCallDelta>>,
}

#[derive(Deserialize)]
struct OaToolCallIn {
    id: String,
    function: OaFunctionIn,
}

#[derive(Deserialize)]
struct OaFunctionIn {
    name: String,
    arguments: String,
}

#[derive(Deserialize)]
struct OaToolCallDelta {
    index: u32,
    id: Option<String>,
    function: Option<OaFunctionDelta>,
}

#[derive(Deserialize)]
struct OaFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Deserialize)]
struct OaUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Serialize)]
struct OaEmbedBody {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct OaEmbedResponse {
    data: Vec<OaEmbedItem>,
}

#[derive(Deserialize)]
struct OaEmbedItem {
    embedding: Vec<f32>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn role_to_str(role: &Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

fn map_message(m: Message) -> OaMessage {
    let tool_calls_out: Option<Vec<OaToolCallOut>> = if m.tool_calls.is_empty() {
        None
    } else {
        Some(
            m.tool_calls
                .into_iter()
                .map(|tc| OaToolCallOut {
                    id: tc.id,
                    r#type: "function",
                    function: OaFunctionCall {
                        name: tc.name,
                        arguments: tc.arguments,
                    },
                })
                .collect(),
        )
    };

    OaMessage {
        role: role_to_str(&m.role).to_string(),
        content: m.content,
        tool_calls: tool_calls_out,
        tool_call_id: m.tool_call_id,
        name: m.name,
    }
}

fn map_tools(tools: Vec<ToolDef>) -> Option<Vec<OaTool>> {
    if tools.is_empty() {
        return None;
    }
    Some(
        tools
            .into_iter()
            .map(|t| OaTool {
                r#type: "function",
                function: OaToolFunction {
                    name: t.name,
                    description: t.description,
                    parameters: t.parameters,
                },
            })
            .collect(),
    )
}

fn map_response_format(rf: Option<super::provider::ResponseFormat>) -> Option<Value> {
    rf.map(|f| match f {
        super::provider::ResponseFormat::Text => {
            serde_json::json!({ "type": "text" })
        }
        super::provider::ResponseFormat::JsonObject => {
            serde_json::json!({ "type": "json_object" })
        }
        super::provider::ResponseFormat::JsonSchema { name, schema, strict } => {
            serde_json::json!({
                "type": "json_schema",
                "json_schema": { "name": name, "schema": schema, "strict": strict }
            })
        }
    })
}

async fn check_status(resp: reqwest::Response) -> AppResult<reqwest::Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let excerpt = resp
        .text()
        .await
        .unwrap_or_else(|_| "<unreadable body>".into());
    let excerpt: String = excerpt.chars().take(400).collect();
    Err(AppError::Provider(format!("openai {}: {}", status, excerpt)))
}

// ---------------------------------------------------------------------------
// OpenAi struct
// ---------------------------------------------------------------------------

pub struct OpenAi {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl OpenAi {
    pub fn with_api_key(key: impl Into<String>) -> Self {
        Self {
            api_key: key.into(),
            base_url: "https://api.openai.com".to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base_url = base.into().trim_end_matches('/').to_string();
        self
    }
}

// ---------------------------------------------------------------------------
// LlmProvider impl
// ---------------------------------------------------------------------------

use async_trait::async_trait;
use super::provider::LlmProvider;

#[async_trait]
impl LlmProvider for OpenAi {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn default_model(&self) -> &'static str {
        "gpt-4o-mini"
    }

    fn default_embedding_model(&self) -> &'static str {
        "text-embedding-3-small"
    }

    // -----------------------------------------------------------------------
    // Non-streaming completion
    // -----------------------------------------------------------------------
    async fn complete(&self, req: CompletionRequest) -> AppResult<CompletionResponse> {
        let body = OaCompletionBody {
            model: req.model.clone(),
            messages: req.messages.into_iter().map(map_message).collect(),
            tools: map_tools(req.tools),
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            response_format: map_response_format(req.response_format),
            stream: false,
            stream_options: None,
        };

        let resp = self
            .http
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp = check_status(resp).await?;
        let parsed: OaCompletionResponse = resp.json().await?;

        let choice = parsed.choices.into_iter().next().ok_or_else(|| {
            AppError::Provider("openai: empty choices array".into())
        })?;

        let msg = choice.message.unwrap_or(OaChoiceMessage {
            content: None,
            tool_calls: None,
        });

        let tool_calls: Vec<ToolCall> = msg
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tc| ToolCall {
                id: tc.id,
                name: tc.function.name,
                arguments: tc.function.arguments,
            })
            .collect();

        let usage = parsed.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }).unwrap_or_default();

        Ok(CompletionResponse {
            content: msg.content,
            tool_calls,
            usage,
            finish_reason: choice.finish_reason.unwrap_or_default(),
            model: parsed.model,
        })
    }

    // -----------------------------------------------------------------------
    // Streaming completion
    // -----------------------------------------------------------------------
    async fn stream(
        &self,
        req: CompletionRequest,
    ) -> AppResult<BoxStream<'static, AppResult<StreamChunk>>> {
        let body = OaCompletionBody {
            model: req.model.clone(),
            messages: req.messages.into_iter().map(map_message).collect(),
            tools: map_tools(req.tools),
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            response_format: map_response_format(req.response_format),
            stream: true,
            stream_options: Some(StreamOptions { include_usage: true }),
        };

        let resp = self
            .http
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp = check_status(resp).await?;

        // Consume the byte stream, split on newlines, parse SSE events.
        let byte_stream = resp.bytes_stream();

        // We buffer incomplete lines across chunks.
        let s = stream::unfold(
            (byte_stream, String::new(), false),
            |(mut byte_stream, mut buf, mut done)| async move {
                if done {
                    return None;
                }
                loop {
                    // Try to process lines already in the buffer first.
                    if let Some(nl) = buf.find('\n') {
                        let line = buf[..nl].trim_end_matches('\r').to_string();
                        buf = buf[nl + 1..].to_string();

                        if line.is_empty() {
                            continue;
                        }

                        let data = if let Some(stripped) = line.strip_prefix("data: ") {
                            stripped.trim()
                        } else {
                            continue;
                        };

                        if data == "[DONE]" {
                            done = true;
                            let chunk = StreamChunk::Done {
                                usage: None,
                                finish_reason: "stop".to_string(),
                            };
                            return Some((Ok(chunk), (byte_stream, buf, done)));
                        }

                        // Parse JSON event
                        match serde_json::from_str::<OaCompletionResponse>(data) {
                            Err(e) => {
                                return Some((
                                    Err(AppError::Provider(format!(
                                        "openai stream parse error: {e}"
                                    ))),
                                    (byte_stream, buf, done),
                                ));
                            }
                            Ok(event) => {
                                // Check if this is a usage-only event (choices empty)
                                if event.choices.is_empty() {
                                    if let Some(u) = event.usage {
                                        let chunk = StreamChunk::Done {
                                            usage: Some(Usage {
                                                prompt_tokens: u.prompt_tokens,
                                                completion_tokens: u.completion_tokens,
                                                total_tokens: u.total_tokens,
                                            }),
                                            finish_reason: String::new(),
                                        };
                                        return Some((Ok(chunk), (byte_stream, buf, done)));
                                    }
                                    continue;
                                }

                                let choice = &event.choices[0];
                                let finish_reason =
                                    choice.finish_reason.clone().unwrap_or_default();

                                if let Some(delta) = &choice.delta {
                                    // Text delta
                                    if let Some(text) = &delta.content {
                                        if !text.is_empty() {
                                            return Some((
                                                Ok(StreamChunk::Text(text.clone())),
                                                (byte_stream, buf, done),
                                            ));
                                        }
                                    }

                                    // Tool call delta
                                    if let Some(tc_deltas) = &delta.tool_calls {
                                        for tcd in tc_deltas {
                                            let id = tcd.id.clone().unwrap_or_default();
                                            let name = tcd
                                                .function
                                                .as_ref()
                                                .and_then(|f| f.name.clone());
                                            let args_delta = tcd
                                                .function
                                                .as_ref()
                                                .and_then(|f| f.arguments.clone())
                                                .unwrap_or_default();
                                            return Some((
                                                Ok(StreamChunk::ToolCallDelta {
                                                    id,
                                                    index: tcd.index,
                                                    name,
                                                    arguments_delta: args_delta,
                                                }),
                                                (byte_stream, buf, done),
                                            ));
                                        }
                                    }
                                }

                                // finish_reason set → emit Done
                                if !finish_reason.is_empty() {
                                    let usage = event.usage.map(|u| Usage {
                                        prompt_tokens: u.prompt_tokens,
                                        completion_tokens: u.completion_tokens,
                                        total_tokens: u.total_tokens,
                                    });
                                    done = true;
                                    return Some((
                                        Ok(StreamChunk::Done { usage, finish_reason }),
                                        (byte_stream, buf, done),
                                    ));
                                }

                                // Empty delta with no finish_reason: loop for next chunk
                                continue;
                            }
                        }
                    }

                    // Need more bytes from the network.
                    match byte_stream.next().await {
                        None => {
                            // Stream ended without [DONE]; emit Done anyway.
                            done = true;
                            return Some((
                                Ok(StreamChunk::Done {
                                    usage: None,
                                    finish_reason: "stop".to_string(),
                                }),
                                (byte_stream, buf, done),
                            ));
                        }
                        Some(Err(e)) => {
                            return Some((Err(AppError::Http(e)), (byte_stream, buf, done)));
                        }
                        Some(Ok(bytes)) => {
                            buf.push_str(&String::from_utf8_lossy(&bytes));
                            // Loop back to try processing the buffer.
                        }
                    }
                }
            },
        );

        Ok(Box::pin(s))
    }

    // -----------------------------------------------------------------------
    // Embeddings
    // -----------------------------------------------------------------------
    async fn embed(&self, model: Option<String>, inputs: Vec<String>) -> AppResult<Vec<Vec<f32>>> {
        let model_str = model.unwrap_or_else(|| self.default_embedding_model().to_string());

        let body = OaEmbedBody {
            model: model_str,
            input: inputs,
        };

        let resp = self
            .http
            .post(format!("{}/v1/embeddings", self.base_url))
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let resp = check_status(resp).await?;
        let parsed: OaEmbedResponse = resp.json().await?;

        Ok(parsed.data.into_iter().map(|item| item.embedding).collect())
    }
}
