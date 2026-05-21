use keyring::Entry;

use crate::error::{AppError, AppResult};

const SERVICE: &str = "com.retposto.app";

fn imap_account_key(email: &str) -> String {
    format!("imap:{email}")
}

/// Store an IMAP password for `email` in the system keychain.
pub fn save_imap_password(email: &str, password: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, &imap_account_key(email))
        .map_err(|e| AppError::Auth(format!("keyring entry creation failed: {e}")))?;
    entry
        .set_password(password)
        .map_err(|e| AppError::Auth(format!("keyring set failed for {email}: {e}")))?;
    Ok(())
}

/// Load an IMAP password for `email` from the system keychain.
/// Returns `None` if no entry exists yet (not an error).
pub fn load_imap_password(email: &str) -> AppResult<Option<String>> {
    let entry = Entry::new(SERVICE, &imap_account_key(email))
        .map_err(|e| AppError::Auth(format!("keyring entry creation failed: {e}")))?;
    match entry.get_password() {
        Ok(pw) => Ok(Some(pw)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Auth(format!(
            "keyring get failed for {email}: {e}"
        ))),
    }
}

/// Delete an IMAP password for `email` from the system keychain.
/// Silently succeeds if no entry exists.
pub fn delete_imap_password(email: &str) -> AppResult<()> {
    let entry = Entry::new(SERVICE, &imap_account_key(email))
        .map_err(|e| AppError::Auth(format!("keyring entry creation failed: {e}")))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Auth(format!(
            "keyring delete failed for {email}: {e}"
        ))),
    }
}
