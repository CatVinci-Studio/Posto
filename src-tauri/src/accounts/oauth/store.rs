use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::accounts::provider::ProviderKind;
use crate::error::{AppError, AppResult};

use super::flow::OAuthTokens;

// ---------------------------------------------------------------------------
// Keyring helpers
// ---------------------------------------------------------------------------

const SERVICE: &str = "com.catvinci.posto";

fn keyring_account(provider: ProviderKind, email: &str) -> String {
    format!("oauth:{:?}:{}", provider, email).to_lowercase()
}

/// Persist tokens for an account.  The full `OAuthTokens` struct is stored as
/// JSON; sensitive fields live inside the OS keychain.
pub fn save_tokens(provider: ProviderKind, account_email: &str, tokens: &OAuthTokens) -> AppResult<()> {
    let account_key = keyring_account(provider, account_email);
    let json = serde_json::to_string(tokens)?;
    keyring::Entry::new(SERVICE, &account_key)
        .map_err(|e| AppError::Auth(e.to_string()))?
        .set_password(&json)
        .map_err(|e| AppError::Auth(e.to_string()))?;
    Ok(())
}

/// Load tokens for an account.  Returns `None` if no entry exists.
pub fn load_tokens(provider: ProviderKind, account_email: &str) -> AppResult<Option<OAuthTokens>> {
    let account_key = keyring_account(provider, account_email);
    let entry = keyring::Entry::new(SERVICE, &account_key)
        .map_err(|e| AppError::Auth(e.to_string()))?;
    match entry.get_password() {
        Ok(json) => {
            let tokens: OAuthTokens = serde_json::from_str(&json)?;
            Ok(Some(tokens))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Auth(e.to_string())),
    }
}

/// Delete stored tokens for an account (used on sign-out).
pub fn delete_tokens(provider: ProviderKind, account_email: &str) -> AppResult<()> {
    let account_key = keyring_account(provider, account_email);
    let entry = keyring::Entry::new(SERVICE, &account_key)
        .map_err(|e| AppError::Auth(e.to_string()))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Auth(e.to_string())),
    }
}

/// Returns `true` when tokens exist in the keyring for the given account.
pub fn has_tokens(provider: ProviderKind, account_email: &str) -> AppResult<bool> {
    Ok(load_tokens(provider, account_email)?.is_some())
}

/// Load tokens and, if the access token is within `buffer_secs` of expiry,
/// silently refresh + persist new tokens. Returns a usable `OAuthTokens`.
///
/// Used by the IMAP XOAUTH2 path so background syncs do not stall when the
/// 1-hour access token expires.
pub async fn get_valid_tokens(
    flow: &super::flow::OAuthFlow,
    provider: ProviderKind,
    account_email: &str,
    buffer_secs: i64,
) -> AppResult<OAuthTokens> {
    let stored = load_tokens(provider, account_email)?
        .ok_or_else(|| AppError::Auth(format!("no OAuth tokens for {account_email}")))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    if stored.expires_at - buffer_secs > now {
        return Ok(stored);
    }

    let refresh = stored
        .refresh_token
        .clone()
        .ok_or_else(|| AppError::Auth("token expired and no refresh_token present".to_string()))?;

    let mut new_tokens = flow.refresh(provider, refresh).await?;
    // Refresh responses sometimes omit refresh_token — keep the old one.
    if new_tokens.refresh_token.is_none() {
        new_tokens.refresh_token = stored.refresh_token;
    }
    save_tokens(provider, account_email, &new_tokens)?;
    Ok(new_tokens)
}

// ---------------------------------------------------------------------------
// In-process pending-state store
// ---------------------------------------------------------------------------

/// Value stored while we await the callback: the provider that was requested
/// plus the PKCE verifier that must be sent to the token endpoint.
pub type PendingEntry = (ProviderKind, String); // (provider, verifier)

/// Managed Tauri state that maps OAuth `state` strings to their pending
/// exchange data.  Consumed exactly once when the callback arrives.
#[derive(Default)]
pub struct PendingStore(pub Arc<Mutex<HashMap<String, PendingEntry>>>);

impl PendingStore {
    pub async fn insert(&self, state: String, entry: PendingEntry) {
        self.0.lock().await.insert(state, entry);
    }

    /// Remove and return the entry for `state`, or `None` if not found /
    /// already consumed (possible replay attack).
    pub async fn take(&self, state: &str) -> Option<PendingEntry> {
        self.0.lock().await.remove(state)
    }
}
