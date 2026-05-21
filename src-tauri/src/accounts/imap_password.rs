use std::time::Duration;

use async_imap::Client;
use async_native_tls::TlsConnector;
use futures::TryStreamExt;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};
use tracing::{debug, warn};

use crate::accounts::provider::{Encryption, FolderInfo, ImapConfig, ParsedEmail};
use crate::error::{AppError, AppResult};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ImapCredentials {
    pub username: String,
    pub password: String,
}

// ---------------------------------------------------------------------------
// Internal helper: open an authenticated IMAP session
// ---------------------------------------------------------------------------

/// `async-imap 0.10` is built on `futures::io` traits, while Tokio's
/// `TcpStream` exposes `tokio::io`. We bridge with `tokio_util::compat::Compat`.
type ImapStream = async_native_tls::TlsStream<Compat<TcpStream>>;

async fn open_session(
    imap: &ImapConfig,
    creds: &ImapCredentials,
) -> AppResult<async_imap::Session<ImapStream>> {
    let addr = format!("{}:{}", imap.host, imap.port);
    debug!("IMAP connecting to {addr} (encryption={:?})", imap.encryption);

    match imap.encryption {
        Encryption::Tls => {
            let tcp = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
                .await
                .map_err(|_| AppError::Provider(format!("timed out connecting to {addr}")))?
                .map_err(|e| AppError::Provider(format!("TCP connect failed: {e}")))?;

            // Tokio AsyncRead/Write -> futures-io AsyncRead/Write.
            let stream = tcp.compat();

            let tls = TlsConnector::new();
            let tls_stream = timeout(
                CONNECT_TIMEOUT,
                tls.connect(&imap.host, stream),
            )
            .await
            .map_err(|_| AppError::Provider("timed out during TLS handshake".to_string()))?
            .map_err(|e| AppError::Provider(format!("TLS handshake failed: {e}")))?;

            let client = Client::new(tls_stream);
            let session = timeout(
                CONNECT_TIMEOUT,
                client.login(&creds.username, &creds.password),
            )
            .await
            .map_err(|_| AppError::Auth("timed out during IMAP login".to_string()))?
            .map_err(|(e, _client)| AppError::Auth(format!("IMAP login failed: {e}")))?;

            debug!("IMAP session established for {}", creds.username);
            Ok(session)
        }

        Encryption::StartTls => {
            // TODO: implement STARTTLS upgrade path.
            // async-imap does not expose a first-class STARTTLS helper; the
            // approach is: connect plain → send STARTTLS command →
            // upgrade the stream → re-wrap in Client → login.
            // For now we return a descriptive error so callers can surface it.
            warn!("STARTTLS requested but not yet implemented; use port 993 / TLS instead");
            Err(AppError::Provider(
                "STARTTLS not implemented yet — please use implicit TLS (port 993)".to_string(),
            ))
        }

        Encryption::None => {
            // TODO: plain-text IMAP (port 143). Not needed for first-class
            // providers, but required for some self-hosted setups.
            Err(AppError::Provider(
                "Plain-text IMAP (no encryption) is not implemented yet".to_string(),
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Login then immediately logout to verify credentials.
pub async fn test_connection(imap: &ImapConfig, creds: &ImapCredentials) -> AppResult<()> {
    let mut session = open_session(imap, creds).await?;
    session
        .logout()
        .await
        .map_err(|e| AppError::Provider(format!("IMAP logout error: {e}")))?;
    debug!("test_connection succeeded for {}", creds.username);
    Ok(())
}

/// List all mailbox folders visible to the authenticated user.
pub async fn fetch_folders(
    imap: &ImapConfig,
    creds: &ImapCredentials,
) -> AppResult<Vec<FolderInfo>> {
    let mut session = open_session(imap, creds).await?;

    let mailboxes = timeout(CONNECT_TIMEOUT, session.list(Some(""), Some("*")))
        .await
        .map_err(|_| AppError::Provider("timed out listing folders".to_string()))?
        .map_err(|e| AppError::Provider(format!("IMAP LIST failed: {e}")))?;

    // Collect the stream of Name responses.
    let names: Vec<_> = mailboxes
        .try_collect()
        .await
        .map_err(|e| AppError::Provider(format!("IMAP LIST stream error: {e}")))?;

    let folders = names
        .into_iter()
        .map(|n| {
            let attrs: Vec<String> = n
                .attributes()
                .iter()
                .map(|a| format!("{a:?}"))
                .collect();
            let path = n.name().to_string();
            FolderInfo {
                name: path.clone(),
                imap_path: path,
                attributes: attrs,
            }
        })
        .collect();

    session
        .logout()
        .await
        .map_err(|e| AppError::Provider(format!("IMAP logout error: {e}")))?;

    Ok(folders)
}

/// Return the most-recent `limit` UIDs in `folder_path`, sorted descending.
pub async fn fetch_recent_uids(
    imap: &ImapConfig,
    creds: &ImapCredentials,
    folder_path: &str,
    limit: u32,
) -> AppResult<Vec<u32>> {
    let mut session = open_session(imap, creds).await?;

    timeout(CONNECT_TIMEOUT, session.select(folder_path))
        .await
        .map_err(|_| AppError::Provider(format!("timed out selecting folder {folder_path}")))?
        .map_err(|e| AppError::Provider(format!("IMAP SELECT failed: {e}")))?;

    // UID SEARCH ALL returns matching UIDs.
    let uid_set = timeout(CONNECT_TIMEOUT, session.uid_search("ALL"))
        .await
        .map_err(|_| AppError::Provider("timed out during UID SEARCH".to_string()))?
        .map_err(|e| AppError::Provider(format!("UID SEARCH failed: {e}")))?;

    let mut uids: Vec<u32> = uid_set.into_iter().collect();
    uids.sort_unstable_by(|a, b| b.cmp(a)); // descending (newest first)
    uids.truncate(limit as usize);

    session
        .logout()
        .await
        .map_err(|e| AppError::Provider(format!("IMAP logout error: {e}")))?;

    Ok(uids)
}

/// Fetch and parse a single message by UID.
pub async fn fetch_message(
    imap: &ImapConfig,
    creds: &ImapCredentials,
    folder_path: &str,
    uid: u32,
) -> AppResult<ParsedEmail> {
    let mut session = open_session(imap, creds).await?;

    timeout(CONNECT_TIMEOUT, session.select(folder_path))
        .await
        .map_err(|_| AppError::Provider(format!("timed out selecting folder {folder_path}")))?
        .map_err(|e| AppError::Provider(format!("IMAP SELECT failed: {e}")))?;

    let fetch_stream = timeout(
        CONNECT_TIMEOUT,
        session.uid_fetch(uid.to_string(), "(BODY[] FLAGS INTERNALDATE)"),
    )
    .await
    .map_err(|_| AppError::Provider(format!("timed out fetching UID {uid}")))?
    .map_err(|e| AppError::Provider(format!("UID FETCH failed: {e}")))?;

    let fetched: Vec<_> = fetch_stream
        .try_collect()
        .await
        .map_err(|e| AppError::Provider(format!("UID FETCH stream error: {e}")))?;

    let fetch = fetched
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound(format!("UID {uid} not found in {folder_path}")))?;

    let raw = fetch
        .body()
        .ok_or_else(|| AppError::Provider(format!("no BODY[] in fetch response for UID {uid}")))?;

    let size = raw.len();

    // Parse with mail-parser
    let parsed = mail_parser::MessageParser::default()
        .parse(raw)
        .ok_or_else(|| AppError::Provider(format!("failed to parse message UID {uid}")))?;

    // ---- Extract fields ----

    let message_id = parsed.message_id().map(|s| s.to_string());

    let (from_addr, from_name) = parsed
        .from()
        .and_then(|al| al.first())
        .map(|addr| {
            (
                addr.address().map(|s| s.to_string()),
                addr.name().map(|s| s.to_string()),
            )
        })
        .unwrap_or((None, None));

    let to: Vec<String> = parsed
        .to()
        .map(|al| {
            al.iter()
                .filter_map(|a| a.address().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let cc: Vec<String> = parsed
        .cc()
        .map(|al| {
            al.iter()
                .filter_map(|a| a.address().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let subject = parsed.subject().map(|s| s.to_string());

    let date = parsed.date().map(|d| d.to_timestamp());

    let body_text = parsed.body_text(0).map(|s| s.into_owned());
    let body_html = parsed.body_html(0).map(|s| s.into_owned());

    let snippet = body_text
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(200)
        .collect();

    let has_attachments = parsed.attachment_count() > 0;

    session
        .logout()
        .await
        .map_err(|e| AppError::Provider(format!("IMAP logout error: {e}")))?;

    Ok(ParsedEmail {
        message_id,
        from_addr,
        from_name,
        to,
        cc,
        subject,
        date,
        body_text,
        body_html,
        snippet,
        has_attachments,
        size,
    })
}
