use tauri::Emitter;
use tracing::{info, warn};

use crate::accounts::provider::ProviderKind;

use super::flow::{AuthInitiation, OAuthFlow, OAuthTokens};
use super::store::{self, PendingStore};

// ---------------------------------------------------------------------------
// begin_oauth_login
// ---------------------------------------------------------------------------

/// Kick off an OAuth flow for the given provider:
///   1. Build the authorization URL + PKCE bundle.
///   2. Record the pending state so the callback can retrieve the verifier.
///   3. Open the URL in the system browser.
///   4. Return `AuthInitiation` so the UI can show a "waiting…" screen.
#[tauri::command]
pub async fn begin_oauth_login(
    provider: ProviderKind,
    app: tauri::AppHandle,
    flow: tauri::State<'_, std::sync::Arc<OAuthFlow>>,
    pending: tauri::State<'_, PendingStore>,
) -> Result<AuthInitiation, String> {
    let initiation = flow.begin(provider).map_err(|e| e.to_string())?;

    pending
        .insert(
            initiation.state.clone(),
            (provider, initiation.verifier.clone()),
        )
        .await;

    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_url(&initiation.auth_url, None::<&str>)
        .map_err(|e| e.to_string())?;

    info!(provider = ?provider, "OAuth browser login opened");

    Ok(initiation)
}

// ---------------------------------------------------------------------------
// handle_oauth_callback
// ---------------------------------------------------------------------------

/// Handle the deep-link callback URL `posto://oauth/callback?code=…&state=…`.
///
/// Called either directly by the deep-link handler in `lib.rs` OR by the
/// frontend (whichever receives the URL first).
#[tauri::command]
pub async fn handle_oauth_callback(
    url: String,
    flow: tauri::State<'_, std::sync::Arc<OAuthFlow>>,
    pending: tauri::State<'_, PendingStore>,
    app: tauri::AppHandle,
) -> Result<OAuthTokens, String> {
    // Parse query parameters from the deep-link URL.
    let parsed = url::Url::parse(&url).map_err(|e| format!("invalid callback URL: {e}"))?;
    let params: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();

    // Surface OAuth errors returned by the provider (e.g. user cancelled).
    if let Some(err) = params.get("error") {
        let desc = params
            .get("error_description")
            .map(|s| s.as_str())
            .unwrap_or("");
        warn!(error = %err, description = %desc, "OAuth callback returned error");
        return Err(format!("OAuth error: {err} — {desc}"));
    }

    let code = params
        .get("code")
        .ok_or_else(|| "callback URL missing 'code'".to_owned())?
        .clone();

    let state = params
        .get("state")
        .ok_or_else(|| "callback URL missing 'state'".to_owned())?
        .clone();

    // Retrieve and consume the pending entry (CSRF check + verifier retrieval).
    let (provider, verifier) = pending
        .take(&state)
        .await
        .ok_or_else(|| "unknown or already-used OAuth state — possible CSRF".to_owned())?;

    // Exchange the authorization code for tokens.
    let tokens = flow
        .exchange_code(provider, code, verifier)
        .await
        .map_err(|e| e.to_string())?;

    // If the provider returned an id_token we could decode the email; for now
    // we persist via the caller who will supply the email once known.
    // Emit an event so the frontend can react immediately.
    app.emit("oauth:tokens_ready", serde_json::json!({
        "provider": provider,
        "tokens": tokens,
    }))
    .map_err(|e| e.to_string())?;

    info!(provider = ?provider, "OAuth token exchange successful");

    Ok(tokens)
}

// ---------------------------------------------------------------------------
// refresh_oauth_tokens
// ---------------------------------------------------------------------------

/// Refresh the access token for an already-authenticated account.
#[tauri::command]
pub async fn refresh_oauth_tokens(
    provider: ProviderKind,
    account_email: String,
    flow: tauri::State<'_, std::sync::Arc<OAuthFlow>>,
) -> Result<OAuthTokens, String> {
    let stored = store::load_tokens(provider, &account_email)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("no stored tokens for {account_email}"))?;

    let refresh_token = stored
        .refresh_token
        .ok_or_else(|| "stored tokens have no refresh_token".to_owned())?;

    let new_tokens = flow
        .refresh(provider, refresh_token)
        .await
        .map_err(|e| e.to_string())?;

    store::save_tokens(provider, &account_email, &new_tokens).map_err(|e| e.to_string())?;

    info!(provider = ?provider, account = %account_email, "Access token refreshed");

    Ok(new_tokens)
}

// ---------------------------------------------------------------------------
// has_oauth_tokens
// ---------------------------------------------------------------------------

/// Returns `true` when the keyring contains tokens for the given account.
#[tauri::command]
pub fn has_oauth_tokens(
    provider: ProviderKind,
    account_email: String,
) -> Result<bool, String> {
    store::has_tokens(provider, &account_email).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// clear_oauth_tokens
// ---------------------------------------------------------------------------

/// Remove stored tokens from the keyring (sign-out / revoke).
#[tauri::command]
pub async fn clear_oauth_tokens(
    provider: ProviderKind,
    account_email: String,
) -> Result<(), String> {
    store::delete_tokens(provider, &account_email).map_err(|e| e.to_string())?;
    info!(provider = ?provider, account = %account_email, "OAuth tokens cleared");
    Ok(())
}
