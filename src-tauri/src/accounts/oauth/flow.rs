use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

use crate::accounts::provider::{AuthMethod, ProviderKind};
use crate::accounts::provider_catalog;
use crate::error::{AppError, AppResult};

use super::pkce;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    pub token_type: String,
    /// Unix timestamp (seconds) at which the access token expires.
    pub expires_at: i64,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInitiation {
    pub provider: ProviderKind,
    /// The URL the user (or the app) opens in the system browser.
    pub auth_url: String,
    /// The opaque CSRF state value; stored in `PendingStore` until callback.
    pub state: String,
    /// The PKCE verifier; also stored in `PendingStore` until callback.
    pub verifier: String,
}

// ---------------------------------------------------------------------------
// Redirect URI used for all providers
// ---------------------------------------------------------------------------

const REDIRECT_URI: &str = "retposto://oauth/callback";

// ---------------------------------------------------------------------------
// Client-ID resolution
// ---------------------------------------------------------------------------

fn google_client_id() -> String {
    // TODO: register a Google Cloud OAuth 2.0 client and set GOOGLE_OAUTH_CLIENT_ID
    std::env::var("GOOGLE_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "PLACEHOLDER_GOOGLE_CLIENT_ID".to_owned())
}

fn microsoft_client_id() -> String {
    // TODO: register a Microsoft Entra app and set MICROSOFT_OAUTH_CLIENT_ID
    std::env::var("MICROSOFT_OAUTH_CLIENT_ID")
        .unwrap_or_else(|_| "PLACEHOLDER_MICROSOFT_CLIENT_ID".to_owned())
}

// ---------------------------------------------------------------------------
// Token-endpoint response shape
// ---------------------------------------------------------------------------

/// Raw JSON shape returned by both Google and Microsoft token endpoints.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    token_type: Option<String>,
    /// Lifetime in seconds; may be absent on refresh responses.
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    scope: Option<String>,
}

// ---------------------------------------------------------------------------
// OAuthFlow
// ---------------------------------------------------------------------------

pub struct OAuthFlow {
    client_id_google: String,
    client_id_microsoft: String,
    http: reqwest::Client,
}

impl OAuthFlow {
    /// Construct from environment variables (or compile-time placeholders).
    /// Call once at startup and store in Tauri managed state.
    pub fn from_env() -> Self {
        Self {
            client_id_google: google_client_id(),
            client_id_microsoft: microsoft_client_id(),
            http: reqwest::Client::new(),
        }
    }

    fn client_id_for(&self, provider: ProviderKind) -> AppResult<&str> {
        match provider {
            ProviderKind::Gmail => Ok(&self.client_id_google),
            ProviderKind::Outlook => Ok(&self.client_id_microsoft),
            other => Err(AppError::Auth(format!(
                "provider {:?} does not use OAuth 2.0",
                other
            ))),
        }
    }

    // -----------------------------------------------------------------------
    // begin()
    // -----------------------------------------------------------------------

    /// Build an authorization URL and return everything the caller needs to
    /// track the pending exchange.
    pub fn begin(&self, provider: ProviderKind) -> AppResult<AuthInitiation> {
        let cfg = provider_catalog::config(provider);

        let (auth_url_base, scopes) = match cfg.auth_method {
            AuthMethod::Oauth2 {
                auth_url, scopes, ..
            } => (auth_url, scopes),
            _ => {
                return Err(AppError::Auth(format!(
                    "provider {:?} does not use OAuth 2.0",
                    provider
                )))
            }
        };

        let client_id = self.client_id_for(provider)?;
        let pkce = pkce::generate();
        let scope_str = scopes.join(" ");

        let mut params = vec![
            ("client_id", client_id.to_owned()),
            ("redirect_uri", REDIRECT_URI.to_owned()),
            ("response_type", "code".to_owned()),
            ("scope", scope_str),
            ("state", pkce.state.clone()),
            ("code_challenge", pkce.challenge.clone()),
            ("code_challenge_method", "S256".to_owned()),
        ];

        match provider {
            ProviderKind::Gmail => {
                params.push(("access_type", "offline".to_owned()));
                params.push(("prompt", "consent".to_owned()));
            }
            ProviderKind::Outlook => {
                params.push(("response_mode", "query".to_owned()));
            }
            _ => {}
        }

        let query = serde_urlencoded::to_string(&params)
            .map_err(|e| AppError::Other(e.to_string()))?;

        let auth_url = format!("{}?{}", auth_url_base, query);

        info!(provider = ?provider, "OAuth flow initiated");

        Ok(AuthInitiation {
            provider,
            auth_url,
            state: pkce.state,
            verifier: pkce.verifier,
        })
    }

    // -----------------------------------------------------------------------
    // exchange_code()
    // -----------------------------------------------------------------------

    pub async fn exchange_code(
        &self,
        provider: ProviderKind,
        code: String,
        verifier: String,
    ) -> AppResult<OAuthTokens> {
        let cfg = provider_catalog::config(provider);
        let token_url = match cfg.auth_method {
            AuthMethod::Oauth2 { token_url, .. } => token_url,
            _ => {
                return Err(AppError::Auth(format!(
                    "provider {:?} does not use OAuth 2.0",
                    provider
                )))
            }
        };

        let client_id = self.client_id_for(provider)?;

        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", &code),
            ("code_verifier", &verifier),
            ("redirect_uri", REDIRECT_URI),
        ];

        info!(provider = ?provider, "Exchanging authorization code for tokens");

        let resp = self
            .http
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(AppError::Http)?;

        self.parse_token_response(provider, resp).await
    }

    // -----------------------------------------------------------------------
    // refresh()
    // -----------------------------------------------------------------------

    pub async fn refresh(
        &self,
        provider: ProviderKind,
        refresh_token: String,
    ) -> AppResult<OAuthTokens> {
        let cfg = provider_catalog::config(provider);
        let (token_url, scopes) = match cfg.auth_method {
            AuthMethod::Oauth2 {
                token_url, scopes, ..
            } => (token_url, scopes),
            _ => {
                return Err(AppError::Auth(format!(
                    "provider {:?} does not use OAuth 2.0",
                    provider
                )))
            }
        };

        let client_id = self.client_id_for(provider)?;
        let scope_str = scopes.join(" ");

        // Microsoft requires the scope echoed back; Google ignores it but it
        // does no harm to include.
        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", &refresh_token),
            ("scope", &scope_str),
        ];

        info!(provider = ?provider, "Refreshing access token");

        let resp = self
            .http
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(AppError::Http)?;

        self.parse_token_response(provider, resp).await
    }

    // -----------------------------------------------------------------------
    // Shared response parser
    // -----------------------------------------------------------------------

    async fn parse_token_response(
        &self,
        provider: ProviderKind,
        resp: reqwest::Response,
    ) -> AppResult<OAuthTokens> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            let excerpt: String = body.chars().take(500).collect();
            warn!(provider = ?provider, status = %status, "Token endpoint returned error");
            return Err(AppError::Auth(format!("{}: {}", status, excerpt)));
        }

        let raw: TokenResponse = resp.json().await.map_err(AppError::Http)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let expires_at = now + raw.expires_in.unwrap_or(3600) as i64;

        Ok(OAuthTokens {
            access_token: raw.access_token,
            refresh_token: raw.refresh_token,
            id_token: raw.id_token,
            token_type: raw.token_type.unwrap_or_else(|| "Bearer".to_owned()),
            expires_at,
            scope: raw.scope,
        })
    }
}
