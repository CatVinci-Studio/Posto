use sqlx::SqlitePool;

use crate::llm::{self, LlmProvider, OpenAi};
use crate::storage::models::Translation;
use crate::storage::queries;

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Translate a message body to the given target language.
///
/// Checks the `translations` cache first.  On a miss, calls the LLM,
/// persists the result, and returns the translated text.
///
/// `target_lang` is an ISO-639-1 code (e.g. "zh", "en", "ja").
#[tauri::command]
pub async fn translate_message(
    message_id: i64,
    target_lang: String,
    pool: tauri::State<'_, SqlitePool>,
) -> Result<String, String> {
    // 1. Check cache.
    if let Some(cached) = queries::get_translation(&pool, message_id, &target_lang)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(cached);
    }

    // 2. Load message body.
    let msg = queries::get_message(&pool, message_id)
        .await
        .map_err(|e| e.to_string())?;

    let body: String = msg
        .body_text
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(8000)
        .collect();

    if body.is_empty() {
        return Err("message has no text body to translate".into());
    }

    // 3. Load LLM.
    let api_key = llm::secrets::load_openai_api_key()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no LLM provider configured".to_string())?;
    let llm_provider = OpenAi::with_api_key(api_key);

    // 4. Translate.
    let translated = llm_provider
        .translate(&body, &target_lang)
        .await
        .map_err(|e| e.to_string())?;

    // 5. Persist.
    let now = chrono::Utc::now().timestamp();
    let t = Translation {
        message_id,
        target_lang: target_lang.clone(),
        body_text: Some(translated.clone()),
        body_html: None,
        model: Some(llm_provider.default_model().to_string()),
        created_at: now,
    };
    queries::insert_translation(&pool, &t)
        .await
        .map_err(|e| e.to_string())?;

    Ok(translated)
}
