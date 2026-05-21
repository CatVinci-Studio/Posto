// ChatGPT OAuth (PKCE) — modeled after Codex CLI's login flow against auth.openai.com.
// V1 STUB: real implementation lands once endpoints and ToS posture are confirmed.

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
    pub expires_at: i64,  // unix seconds
}

pub async fn begin_login() -> crate::error::AppResult<AuthInitiation> {
    Err(crate::error::AppError::Other(
        "ChatGPT OAuth not yet implemented; use API key path".into(),
    ))
}

pub async fn complete_login(_code: String, _verifier: String) -> crate::error::AppResult<OAuthTokens> {
    Err(crate::error::AppError::Other(
        "ChatGPT OAuth not yet implemented; use API key path".into(),
    ))
}
