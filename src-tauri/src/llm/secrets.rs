use crate::error::{AppError, AppResult};

const SERVICE: &str = "com.catvinci.posto";

// ---------------------------------------------------------------------------
// OpenAI API key
// ---------------------------------------------------------------------------

pub fn save_openai_api_key(key: &str) -> AppResult<()> {
    keyring::Entry::new(SERVICE, "openai:api_key")
        .map_err(|e| AppError::Auth(e.to_string()))?
        .set_password(key)
        .map_err(|e| AppError::Auth(e.to_string()))?;
    Ok(())
}

pub fn load_openai_api_key() -> AppResult<Option<String>> {
    let entry = keyring::Entry::new(SERVICE, "openai:api_key")
        .map_err(|e| AppError::Auth(e.to_string()))?;
    match entry.get_password() {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Auth(e.to_string())),
    }
}

pub fn delete_openai_api_key() -> AppResult<()> {
    let entry = keyring::Entry::new(SERVICE, "openai:api_key")
        .map_err(|e| AppError::Auth(e.to_string()))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Auth(e.to_string())),
    }
}

// ---------------------------------------------------------------------------
// OpenAI OAuth refresh token
// ---------------------------------------------------------------------------

pub fn save_openai_oauth_refresh(token: &str) -> AppResult<()> {
    keyring::Entry::new(SERVICE, "openai:oauth_refresh")
        .map_err(|e| AppError::Auth(e.to_string()))?
        .set_password(token)
        .map_err(|e| AppError::Auth(e.to_string()))?;
    Ok(())
}

pub fn load_openai_oauth_refresh() -> AppResult<Option<String>> {
    let entry = keyring::Entry::new(SERVICE, "openai:oauth_refresh")
        .map_err(|e| AppError::Auth(e.to_string()))?;
    match entry.get_password() {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Auth(e.to_string())),
    }
}

pub fn delete_openai_oauth_refresh() -> AppResult<()> {
    let entry = keyring::Entry::new(SERVICE, "openai:oauth_refresh")
        .map_err(|e| AppError::Auth(e.to_string()))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Auth(e.to_string())),
    }
}
