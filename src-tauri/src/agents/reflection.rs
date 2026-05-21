use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::{AppError, AppResult};
use crate::llm::provider::{CompletionRequest, LlmProvider, Message, ResponseFormat, Role};
use crate::storage::models::Message as EmailMessage;

use super::action::ActionOutput;
use super::prompts::REFLECTION_SYSTEM;
use super::summary::SummaryOutput;
use super::triage::TriageOutput;

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

/// A single memory candidate proposed by the reflection agent.
/// Re-exported so pipeline.rs and memory::writer can use it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCandidate {
    #[serde(rename = "type")]
    pub r#type: String,
    pub scope: String,
    pub key: Option<String>,
    pub content: String,
    pub importance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionOutput {
    #[serde(default)]
    pub candidates: Vec<MemoryCandidate>,
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Run the reflection agent over the email and the outputs of prior agents.
#[instrument(skip(llm, msg, triage, summary, action), fields(msg_id = ?msg.id))]
pub async fn run(
    llm: &dyn LlmProvider,
    msg: &EmailMessage,
    triage: &TriageOutput,
    summary: &SummaryOutput,
    action: &ActionOutput,
) -> AppResult<ReflectionOutput> {
    let subject = msg.subject.as_deref().unwrap_or("(no subject)");
    let from_name = msg.from_name.as_deref().unwrap_or("");
    let from_addr = msg.from_addr.as_deref().unwrap_or("");

    let body_preview: String = msg
        .snippet
        .as_deref()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            msg.body_text
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(4000)
                .collect()
        });

    // Serialize prior agent outputs compactly for the reflection context.
    let triage_json = serde_json::to_string(triage).unwrap_or_default();
    let summary_json = serde_json::to_string(summary).unwrap_or_default();
    let action_json = serde_json::to_string(action).unwrap_or_default();

    let user_content = format!(
        "Subject: {subject}\n\
         From: {from_name} <{from_addr}>\n\n\
         Email body:\n\
         {body_preview}\n\n\
         --- Prior agent outputs ---\n\
         Triage: {triage_json}\n\
         Summary: {summary_json}\n\
         Action: {action_json}"
    );

    let req = CompletionRequest {
        model: llm.default_model().to_string(),
        messages: vec![
            Message {
                role: Role::System,
                content: Some(REFLECTION_SYSTEM.to_string()),
                tool_calls: vec![],
                tool_call_id: None,
                name: None,
            },
            Message {
                role: Role::User,
                content: Some(user_content),
                tool_calls: vec![],
                tool_call_id: None,
                name: None,
            },
        ],
        tools: vec![],
        temperature: Some(0.3),
        max_tokens: Some(1024),
        response_format: Some(ResponseFormat::JsonObject),
    };

    let resp = llm.complete(req).await?;
    let raw_json = resp
        .content
        .ok_or_else(|| AppError::Provider("reflection: empty response content".into()))?;

    let output: ReflectionOutput =
        serde_json::from_str(&raw_json).map_err(|e| AppError::Json(e))?;

    Ok(output)
}
