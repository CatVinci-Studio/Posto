use std::fmt;
use std::str::FromStr;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Gmail,
    Outlook,
    ICloud,
    Qq,
    Mail163,
    GenericImap,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::Gmail => "gmail",
            ProviderKind::Outlook => "outlook",
            ProviderKind::ICloud => "icloud",
            ProviderKind::Qq => "qq",
            ProviderKind::Mail163 => "mail163",
            ProviderKind::GenericImap => "generic_imap",
        }
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProviderKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gmail" => Ok(ProviderKind::Gmail),
            "outlook" => Ok(ProviderKind::Outlook),
            "icloud" => Ok(ProviderKind::ICloud),
            "qq" => Ok(ProviderKind::Qq),
            "mail163" => Ok(ProviderKind::Mail163),
            "generic_imap" => Ok(ProviderKind::GenericImap),
            other => Err(format!("unknown provider kind: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FolderKind {
    Inbox,
    Sent,
    Drafts,
    Archive,
    Spam,
    Trash,
    Custom,
}

impl FolderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            FolderKind::Inbox => "inbox",
            FolderKind::Sent => "sent",
            FolderKind::Drafts => "drafts",
            FolderKind::Archive => "archive",
            FolderKind::Spam => "spam",
            FolderKind::Trash => "trash",
            FolderKind::Custom => "custom",
        }
    }
}

impl fmt::Display for FolderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for FolderKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "inbox" => Ok(FolderKind::Inbox),
            "sent" => Ok(FolderKind::Sent),
            "drafts" => Ok(FolderKind::Drafts),
            "archive" => Ok(FolderKind::Archive),
            "spam" => Ok(FolderKind::Spam),
            "trash" => Ok(FolderKind::Trash),
            "custom" => Ok(FolderKind::Custom),
            other => Err(format!("unknown folder kind: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Triage,
    Summary,
    Action,
    Draft,
    Rule,
    Reflection,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::Triage => "triage",
            AgentType::Summary => "summary",
            AgentType::Action => "action",
            AgentType::Draft => "draft",
            AgentType::Rule => "rule",
            AgentType::Reflection => "reflection",
        }
    }
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AgentType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "triage" => Ok(AgentType::Triage),
            "summary" => Ok(AgentType::Summary),
            "action" => Ok(AgentType::Action),
            "draft" => Ok(AgentType::Draft),
            "rule" => Ok(AgentType::Rule),
            "reflection" => Ok(AgentType::Reflection),
            other => Err(format!("unknown agent type: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Contact,
    Preference,
    Project,
    Rule,
    Fact,
}

impl MemoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryType::Contact => "contact",
            MemoryType::Preference => "preference",
            MemoryType::Project => "project",
            MemoryType::Rule => "rule",
            MemoryType::Fact => "fact",
        }
    }
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MemoryType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "contact" => Ok(MemoryType::Contact),
            "preference" => Ok(MemoryType::Preference),
            "project" => Ok(MemoryType::Project),
            "rule" => Ok(MemoryType::Rule),
            "fact" => Ok(MemoryType::Fact),
            other => Err(format!("unknown memory type: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Open,
    Done,
    Snoozed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Open => "open",
            TaskStatus::Done => "done",
            TaskStatus::Snoozed => "snoozed",
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(TaskStatus::Open),
            "done" => Ok(TaskStatus::Done),
            "snoozed" => Ok(TaskStatus::Snoozed),
            other => Err(format!("unknown task status: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingOpStatus {
    Pending,
    InProgress,
    Failed,
}

impl PendingOpStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PendingOpStatus::Pending => "pending",
            PendingOpStatus::InProgress => "in_progress",
            PendingOpStatus::Failed => "failed",
        }
    }
}

impl fmt::Display for PendingOpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PendingOpStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(PendingOpStatus::Pending),
            "in_progress" => Ok(PendingOpStatus::InProgress),
            "failed" => Ok(PendingOpStatus::Failed),
            other => Err(format!("unknown pending_op status: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Row structs
// All timestamps are i64 unix seconds; JSON columns are Option<String> (raw
// JSON text) so sqlx can map them directly from SQLite TEXT columns without
// needing a custom Decode impl.  Callers that want structured access can
// parse them with serde_json::from_str.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id:              Option<i64>,
    pub provider:        Option<String>,
    pub email:           Option<String>,
    pub display_name:    Option<String>,
    pub oauth_token_ref: Option<String>,
    pub imap_host:       Option<String>,
    pub imap_port:       Option<i64>,
    pub imap_encryption: Option<String>,
    pub smtp_host:       Option<String>,
    pub smtp_port:       Option<i64>,
    pub smtp_encryption: Option<String>,
    pub status:          Option<String>,
    pub created_at:      i64,
    pub updated_at:      i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Folder {
    pub id:           Option<i64>,
    pub account_id:   i64,
    pub name:         Option<String>,
    pub kind:         Option<String>,
    pub imap_path:    Option<String>,
    pub uid_validity: Option<i64>,
    pub uid_next:     Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id:                Option<i64>,
    pub account_id:        i64,
    pub folder_id:         i64,
    pub uid:               i64,
    pub message_id_header: Option<String>,
    pub thread_id:         Option<String>,
    pub from_addr:         Option<String>,
    pub from_name:         Option<String>,
    /// JSON array of address strings, stored as TEXT in SQLite
    pub to_addrs:          Option<String>,
    /// JSON array of address strings, stored as TEXT in SQLite
    pub cc_addrs:          Option<String>,
    pub subject:           Option<String>,
    pub date:              Option<i64>,
    pub snippet:           Option<String>,
    pub body_text:         Option<String>,
    pub body_html:         Option<String>,
    /// JSON array of flag strings, stored as TEXT in SQLite
    pub flags:             Option<String>,
    /// JSON array of label strings, stored as TEXT in SQLite
    pub labels:            Option<String>,
    pub detected_lang:     Option<String>,
    pub lang_confidence:   Option<f64>,
    pub has_attachments:   Option<i64>,
    pub size:              Option<i64>,
    pub created_at:        i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Attachment {
    pub id:         Option<i64>,
    pub message_id: i64,
    pub filename:   Option<String>,
    pub mime:       Option<String>,
    pub size:       Option<i64>,
    pub content_id: Option<String>,
    pub blob_path:  Option<String>,
    pub downloaded: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct AgentRun {
    pub id:            Option<i64>,
    pub message_id:    Option<i64>,
    pub agent_type:    Option<String>,
    pub input_summary: Option<String>,
    /// JSON value, stored as TEXT in SQLite
    pub output:        Option<String>,
    pub tokens_in:     Option<i64>,
    pub tokens_out:    Option<i64>,
    pub model:         Option<String>,
    pub cost:          Option<f64>,
    pub created_at:    i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id:         Option<i64>,
    pub message_id: Option<i64>,
    pub account_id: Option<i64>,
    pub title:      Option<String>,
    pub due_at:     Option<i64>,
    pub status:     Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Memory {
    pub id:                Option<i64>,
    #[sqlx(rename = "type")]
    pub memory_type:       Option<String>,
    pub scope:             Option<String>,
    pub key:               Option<String>,
    pub content:           Option<String>,
    pub embedding:         Option<Vec<u8>>,
    pub importance:        Option<f64>,
    pub pinned:            Option<i64>,
    pub source_message_id: Option<i64>,
    pub created_at:        i64,
    pub last_used_at:      Option<i64>,
    pub use_count:         Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Translation {
    pub message_id:  i64,
    pub target_lang: String,
    pub body_text:   Option<String>,
    pub body_html:   Option<String>,
    pub model:       Option<String>,
    pub created_at:  i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Rule {
    pub id:           Option<i64>,
    pub name:         Option<String>,
    /// JSON value
    pub trigger:      Option<String>,
    /// JSON value
    pub conditions:   Option<String>,
    /// JSON value
    pub actions:      Option<String>,
    pub enabled:      Option<i64>,
    pub created_by:   Option<String>,
    pub created_at:   i64,
    pub last_fired_at: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct PendingOp {
    pub id:         Option<i64>,
    pub account_id: Option<i64>,
    pub op_type:    Option<String>,
    /// JSON value
    pub payload:    Option<String>,
    pub status:     Option<String>,
    pub retries:    Option<i64>,
    pub last_error: Option<String>,
    pub created_at: i64,
}
