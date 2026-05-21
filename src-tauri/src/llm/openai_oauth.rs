// ChatGPT OAuth (PKCE) — placeholder.
//
// This module is a deliberate stub. OpenAI does not publicly publish an OAuth
// 2.0 client registration program for third-party apps to sign users in with
// their ChatGPT account; Codex CLI uses an internal client ID against
// auth.openai.com that is not authorized for redistribution. Wiring this up in
// Posto without an official partnership would violate the OpenAI ToS.
//
// In its place, Posto uses the API key path: users paste an OpenAI API key
// in Settings → LLM Provider, which is encrypted into the OS keychain via
// `llm::secrets`. That path is feature-complete and covers all calls
// (chat / streaming / embeddings).
//
// If OpenAI publishes a public OAuth scheme later, the implementation pattern
// is the same as `accounts::oauth::flow`: build PKCE bundle, open browser,
// receive code via a deep-link callback, exchange for tokens, persist via
// `llm::secrets`. The stub functions below stay so callers compile.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthInitiation {
    pub auth_url: String,
    pub state: String,
    pub code_verifier: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_at: i64,
}

pub async fn begin_login() -> crate::error::AppResult<AuthInitiation> {
    Err(crate::error::AppError::Other(
        "ChatGPT OAuth is unavailable — use the API key path in Settings → LLM Provider".into(),
    ))
}

pub async fn complete_login(
    _code: String,
    _verifier: String,
) -> crate::error::AppResult<OAuthTokens> {
    Err(crate::error::AppError::Other(
        "ChatGPT OAuth is unavailable — use the API key path in Settings → LLM Provider".into(),
    ))
}
