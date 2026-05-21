use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Gmail,
    Outlook,
    ICloud,
    Qq,
    Mail163,
    GenericImap,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Encryption {
    Tls,      // implicit TLS on connect
    StartTls, // upgrade after connect
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub encryption: Encryption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub encryption: Encryption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum AuthMethod {
    Oauth2 {
        auth_url: String,
        token_url: String,
        scopes: Vec<String>,
    },
    AppPassword, // iCloud
    AuthCode,    // QQ, 163
    Password,    // Generic IMAP
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    pub display_name: &'static str, // English; UI will translate via i18n
    pub auth_method: AuthMethod,
    pub imap: Option<ImapConfig>, // None for providers we'll only access via API (none in scope yet)
    pub smtp: Option<SmtpConfig>,
    pub setup_url: Option<&'static str>, // where users go to generate code/password
    pub setup_guide_key: Option<&'static str>, // i18n key for in-app step-by-step guide
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    pub name: String,
    pub imap_path: String,
    pub attributes: Vec<String>, // \\Inbox, \\Sent, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedAttachment {
    pub filename: String,
    pub mime: Option<String>,
    pub content_id: Option<String>,
    #[serde(skip)]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedEmail {
    pub message_id: Option<String>,
    pub from_addr: Option<String>,
    pub from_name: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: Option<String>,
    pub date: Option<i64>,        // unix seconds
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub snippet: String,          // first ~200 chars of text
    pub has_attachments: bool,
    pub size: usize,
    #[serde(skip)]
    pub attachments: Vec<ParsedAttachment>,
}
