use std::time::Duration;

use async_imap::Client;
use async_native_tls::TlsConnector;
use futures::TryStreamExt;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt};
use tracing::{debug, warn};

use crate::accounts::provider::{Encryption, FolderInfo, ImapConfig, ParsedAttachment, ParsedEmail};
use crate::error::{AppError, AppResult};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ImapCredentials {
    pub username: String,
    pub password: String,
}

/// Auth mode for an IMAP session: classic password (LOGIN) or XOAUTH2.
pub enum ImapAuth {
    Password(ImapCredentials),
    XOAuth2 {
        email: String,
        access_token: String,
    },
}

impl ImapAuth {
    fn username(&self) -> &str {
        match self {
            ImapAuth::Password(c) => &c.username,
            ImapAuth::XOAuth2 { email, .. } => email,
        }
    }
}

async fn open_session_for(
    imap: &ImapConfig,
    auth: &ImapAuth,
) -> AppResult<async_imap::Session<ImapStream>> {
    match auth {
        ImapAuth::Password(creds) => open_session(imap, creds).await,
        ImapAuth::XOAuth2 {
            email,
            access_token,
        } => open_session_xoauth2(imap, email, access_token).await,
    }
}

// ---------------------------------------------------------------------------
// Internal helper: open an authenticated IMAP session
// ---------------------------------------------------------------------------

/// `async-imap 0.10` is built on `futures::io` traits, while Tokio's
/// `TcpStream` exposes `tokio::io`. We bridge with `tokio_util::compat::Compat`.
type ImapStream = async_native_tls::TlsStream<Compat<TcpStream>>;

/// XOAUTH2 authenticator for async-imap (SASL).
///
/// async-imap's Authenticator trait is invoked with the server challenge bytes
/// (typically empty for XOAUTH2 initial response) and expects the base64-encoded
/// SASL string back.
struct XOAuth2Authenticator {
    user: String,
    access_token: String,
}

impl async_imap::Authenticator for &XOAuth2Authenticator {
    type Response = String;
    fn process(&mut self, _challenge: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}

/// Open an XOAUTH2-authenticated IMAP session.
///
/// Used by Gmail and Outlook accounts that authenticated via OAuth 2.0; the
/// IMAP server accepts `AUTHENTICATE XOAUTH2` instead of a plaintext password.
pub async fn open_session_xoauth2(
    imap: &ImapConfig,
    email: &str,
    access_token: &str,
) -> AppResult<async_imap::Session<ImapStream>> {
    let addr = format!("{}:{}", imap.host, imap.port);
    debug!("IMAP XOAUTH2 connecting to {addr}");

    // Only implicit TLS is supported for XOAUTH2 (all OAuth providers use 993).
    if !matches!(imap.encryption, Encryption::Tls) {
        return Err(AppError::Provider(
            "XOAUTH2 requires implicit TLS (port 993)".to_string(),
        ));
    }

    let tcp = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
        .await
        .map_err(|_| AppError::Provider(format!("timed out connecting to {addr}")))?
        .map_err(|e| AppError::Provider(format!("TCP connect failed: {e}")))?;
    let stream = tcp.compat();

    let tls = TlsConnector::new();
    let tls_stream = timeout(CONNECT_TIMEOUT, tls.connect(&imap.host, stream))
        .await
        .map_err(|_| AppError::Provider("timed out during TLS handshake".to_string()))?
        .map_err(|e| AppError::Provider(format!("TLS handshake failed: {e}")))?;

    let client = Client::new(tls_stream);

    let authenticator = XOAuth2Authenticator {
        user: email.to_string(),
        access_token: access_token.to_string(),
    };
    let session = timeout(
        CONNECT_TIMEOUT,
        client.authenticate("XOAUTH2", &authenticator),
    )
    .await
    .map_err(|_| AppError::Auth("timed out during XOAUTH2 auth".to_string()))?
    .map_err(|(e, _client)| AppError::Auth(format!("XOAUTH2 auth failed: {e}")))?;

    debug!("IMAP XOAUTH2 session established for {email}");
    Ok(session)
}

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
            // Plain TCP connect → IMAP greeting → STARTTLS → TLS upgrade → login.
            let tcp = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
                .await
                .map_err(|_| AppError::Provider(format!("timed out connecting to {addr}")))?
                .map_err(|e| AppError::Provider(format!("TCP connect failed: {e}")))?;
            let stream = tcp.compat();

            let mut client = Client::new(stream);

            // Consume the server greeting before issuing any command.
            let _greeting = timeout(CONNECT_TIMEOUT, client.read_response())
                .await
                .map_err(|_| AppError::Provider("timed out reading IMAP greeting".to_string()))?
                .ok_or_else(|| AppError::Provider("server closed before greeting".to_string()))?
                .map_err(|e| AppError::Provider(format!("IMAP greeting error: {e}")))?;

            timeout(
                CONNECT_TIMEOUT,
                client.run_command_and_check_ok("STARTTLS", None),
            )
            .await
            .map_err(|_| AppError::Provider("timed out during STARTTLS".to_string()))?
            .map_err(|e| AppError::Provider(format!("STARTTLS failed: {e}")))?;

            let inner = client.into_inner();

            let tls = TlsConnector::new();
            let tls_stream = timeout(CONNECT_TIMEOUT, tls.connect(&imap.host, inner))
                .await
                .map_err(|_| {
                    AppError::Provider("timed out during STARTTLS handshake".to_string())
                })?
                .map_err(|e| AppError::Provider(format!("STARTTLS handshake failed: {e}")))?;

            let client = Client::new(tls_stream);
            let session = timeout(
                CONNECT_TIMEOUT,
                client.login(&creds.username, &creds.password),
            )
            .await
            .map_err(|_| AppError::Auth("timed out during IMAP login".to_string()))?
            .map_err(|(e, _client)| AppError::Auth(format!("IMAP login failed: {e}")))?;

            debug!("IMAP STARTTLS session established for {}", creds.username);
            Ok(session)
        }

        Encryption::None => {
            // Plain-text IMAP (port 143) is intentionally not supported.
            // All five first-class providers use TLS, and unencrypted IMAP is a
            // security antipattern. Self-hosted users should use STARTTLS.
            warn!("plain-text IMAP requested but not supported");
            Err(AppError::Provider(
                "Plain-text IMAP is not supported — use TLS (port 993) or STARTTLS (port 143)"
                    .to_string(),
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Login then immediately logout to verify credentials.
pub async fn test_connection(imap: &ImapConfig, auth: &ImapAuth) -> AppResult<()> {
    let mut session = open_session_for(imap, auth).await?;
    session
        .logout()
        .await
        .map_err(|e| AppError::Provider(format!("IMAP logout error: {e}")))?;
    debug!("test_connection succeeded for {}", auth.username());
    Ok(())
}

/// List all mailbox folders visible to the authenticated user.
pub async fn fetch_folders(
    imap: &ImapConfig,
    auth: &ImapAuth,
) -> AppResult<Vec<FolderInfo>> {
    let mut session = open_session_for(imap, auth).await?;

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
    auth: &ImapAuth,
    folder_path: &str,
    limit: u32,
) -> AppResult<Vec<u32>> {
    let state = fetch_folder_state(imap, auth, folder_path, limit).await?;
    Ok(state.recent_uids)
}

/// Snapshot of a folder's IMAP state used to detect UIDVALIDITY changes.
#[derive(Debug)]
pub struct FolderState {
    pub uid_validity: u32,
    pub uid_next: u32,
    pub recent_uids: Vec<u32>,
}

/// SELECT the folder, capture UIDVALIDITY + UIDNEXT, then run UID SEARCH ALL.
/// Callers compare `uid_validity` against the local cache to detect server-side
/// re-numbering (RFC 3501 §2.3.1.1) and wipe stale rows when it changes.
pub async fn fetch_folder_state(
    imap: &ImapConfig,
    auth: &ImapAuth,
    folder_path: &str,
    limit: u32,
) -> AppResult<FolderState> {
    let mut session = open_session_for(imap, auth).await?;

    let mailbox = timeout(CONNECT_TIMEOUT, session.select(folder_path))
        .await
        .map_err(|_| AppError::Provider(format!("timed out selecting folder {folder_path}")))?
        .map_err(|e| AppError::Provider(format!("IMAP SELECT failed: {e}")))?;

    let uid_validity = mailbox.uid_validity.unwrap_or(0);
    let uid_next = mailbox.uid_next.unwrap_or(0);

    let uid_set = timeout(CONNECT_TIMEOUT, session.uid_search("ALL"))
        .await
        .map_err(|_| AppError::Provider("timed out during UID SEARCH".to_string()))?
        .map_err(|e| AppError::Provider(format!("UID SEARCH failed: {e}")))?;

    let mut uids: Vec<u32> = uid_set.into_iter().collect();
    uids.sort_unstable_by(|a, b| b.cmp(a));
    uids.truncate(limit as usize);

    session
        .logout()
        .await
        .map_err(|e| AppError::Provider(format!("IMAP logout error: {e}")))?;

    Ok(FolderState {
        uid_validity,
        uid_next,
        recent_uids: uids,
    })
}

// ---------------------------------------------------------------------------
// IDLE
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum IdleEvent {
    /// Server reported a change (EXISTS / RECENT / EXPUNGE).
    Notified,
    /// 29-minute keepalive timer elapsed; caller should re-IDLE.
    Timeout,
}

const IDLE_TIMEOUT_SECS: u64 = 29 * 60;

/// SELECT `folder` and wait inside a single IDLE call. Returns once the server
/// reports new data OR the 29-minute keepalive window elapses. Caller is
/// expected to loop and call this again, doing an incremental sync between
/// returns.
pub async fn idle_wait_once(
    imap: &ImapConfig,
    auth: &ImapAuth,
    folder: &str,
) -> AppResult<IdleEvent> {
    let mut session = open_session_for(imap, auth).await?;
    timeout(CONNECT_TIMEOUT, session.select(folder))
        .await
        .map_err(|_| AppError::Provider(format!("timed out selecting {folder} for IDLE")))?
        .map_err(|e| AppError::Provider(format!("IMAP SELECT failed: {e}")))?;

    let mut idle = session.idle();
    idle.init()
        .await
        .map_err(|e| AppError::Provider(format!("IDLE init failed: {e}")))?;

    let event = {
        let (fut, _stop) = idle.wait_with_timeout(Duration::from_secs(IDLE_TIMEOUT_SECS));
        match fut.await {
            Ok(async_imap::extensions::idle::IdleResponse::NewData(_)) => IdleEvent::Notified,
            Ok(_) => IdleEvent::Timeout,
            Err(e) => return Err(AppError::Provider(format!("IDLE wait failed: {e}"))),
        }
    };

    // Send DONE and reclaim session for clean logout.
    let mut session = idle
        .done()
        .await
        .map_err(|e| AppError::Provider(format!("IDLE done failed: {e}")))?;
    let _ = session.logout().await;

    Ok(event)
}

/// Fetch and parse a single message by UID.
pub async fn fetch_message(
    imap: &ImapConfig,
    auth: &ImapAuth,
    folder_path: &str,
    uid: u32,
) -> AppResult<ParsedEmail> {
    let mut session = open_session_for(imap, auth).await?;

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

    // Extract attachment blobs (already in memory inside the BODY[] we fetched).
    use mail_parser::MimeHeaders;
    let mut attachments: Vec<ParsedAttachment> = Vec::new();
    for att in parsed.attachments() {
        let filename = att
            .attachment_name()
            .map(|s| s.to_string())
            .or_else(|| att.content_id().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("attachment-{}.bin", attachments.len() + 1));

        let mime = att.content_type().map(|ct| match ct.subtype() {
            Some(sub) => format!("{}/{}", ct.ctype(), sub),
            None => ct.ctype().to_string(),
        });
        let content_id = att.content_id().map(|s| s.to_string());

        attachments.push(ParsedAttachment {
            filename,
            mime,
            content_id,
            data: att.contents().to_vec(),
        });
    }

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
        attachments,
    })
}
