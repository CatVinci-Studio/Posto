use super::{
    openai,
    provider::{CompletionRequest, LlmProvider, Message, Role},
    secrets,
};

#[tauri::command]
pub async fn set_openai_api_key(key: String) -> Result<(), String> {
    secrets::save_openai_api_key(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn has_openai_api_key() -> Result<bool, String> {
    Ok(secrets::load_openai_api_key().map_err(|e| e.to_string())?.is_some())
}

#[tauri::command]
pub async fn clear_openai_api_key() -> Result<(), String> {
    secrets::delete_openai_api_key().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_openai_completion(prompt: String) -> Result<String, String> {
    let key = secrets::load_openai_api_key()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no api key set".to_string())?;
    let client = openai::OpenAi::with_api_key(key);
    let resp = client
        .complete(CompletionRequest {
            model: client.default_model().to_string(),
            messages: vec![Message {
                role: Role::User,
                content: Some(prompt),
                tool_calls: vec![],
                tool_call_id: None,
                name: None,
            }],
            tools: vec![],
            temperature: Some(0.2),
            max_tokens: Some(256),
            response_format: None,
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(resp.content.unwrap_or_default())
}
