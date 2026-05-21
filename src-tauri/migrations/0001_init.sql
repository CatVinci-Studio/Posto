-- Retposto initial schema
-- sqlx wraps migrations in a transaction automatically; no BEGIN/COMMIT needed.

CREATE TABLE IF NOT EXISTS accounts (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    provider         TEXT,
    email            TEXT UNIQUE,
    display_name     TEXT,
    oauth_token_ref  TEXT,
    imap_host        TEXT,
    imap_port        INTEGER,
    imap_encryption  TEXT,
    smtp_host        TEXT,
    smtp_port        INTEGER,
    smtp_encryption  TEXT,
    status           TEXT DEFAULT 'active',
    created_at       INTEGER NOT NULL,
    updated_at       INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS folders (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id    INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name          TEXT,
    kind          TEXT,
    imap_path     TEXT,
    uid_validity  INTEGER,
    uid_next      INTEGER,
    UNIQUE(account_id, imap_path)
);

CREATE TABLE IF NOT EXISTS messages (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id        INTEGER NOT NULL REFERENCES accounts(id),
    folder_id         INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    uid               INTEGER NOT NULL,
    message_id_header TEXT,
    thread_id         TEXT,
    from_addr         TEXT,
    from_name         TEXT,
    to_addrs          TEXT,
    cc_addrs          TEXT,
    subject           TEXT,
    date              INTEGER,
    snippet           TEXT,
    body_text         TEXT,
    body_html         TEXT,
    flags             TEXT,
    labels            TEXT,
    detected_lang     TEXT,
    lang_confidence   REAL,
    has_attachments   INTEGER DEFAULT 0,
    size              INTEGER,
    created_at        INTEGER NOT NULL,
    UNIQUE(account_id, folder_id, uid)
);

CREATE TABLE IF NOT EXISTS attachments (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id  INTEGER NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    filename    TEXT,
    mime        TEXT,
    size        INTEGER,
    content_id  TEXT,
    blob_path   TEXT,
    downloaded  INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS agent_runs (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id    INTEGER REFERENCES messages(id),
    agent_type    TEXT,
    input_summary TEXT,
    output        TEXT,
    tokens_in     INTEGER,
    tokens_out    INTEGER,
    model         TEXT,
    cost          REAL,
    created_at    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER REFERENCES messages(id),
    account_id INTEGER REFERENCES accounts(id),
    title      TEXT,
    due_at     INTEGER,
    status     TEXT DEFAULT 'open',
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS memories (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    type              TEXT,
    scope             TEXT,
    key               TEXT,
    content           TEXT,
    embedding         BLOB,
    importance        REAL DEFAULT 0.5,
    pinned            INTEGER DEFAULT 0,
    source_message_id INTEGER,
    created_at        INTEGER NOT NULL,
    last_used_at      INTEGER,
    use_count         INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS translations (
    message_id  INTEGER NOT NULL,
    target_lang TEXT NOT NULL,
    body_text   TEXT,
    body_html   TEXT,
    model       TEXT,
    created_at  INTEGER NOT NULL,
    PRIMARY KEY(message_id, target_lang),
    FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS rules (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT,
    trigger      TEXT,
    conditions   TEXT,
    actions      TEXT,
    enabled      INTEGER DEFAULT 1,
    created_by   TEXT,
    created_at   INTEGER NOT NULL,
    last_fired_at INTEGER
);

CREATE TABLE IF NOT EXISTS pending_ops (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id  INTEGER REFERENCES accounts(id),
    op_type     TEXT,
    payload     TEXT,
    status      TEXT DEFAULT 'pending',
    retries     INTEGER DEFAULT 0,
    last_error  TEXT,
    created_at  INTEGER NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_messages_account_folder_date ON messages(account_id, folder_id, date DESC);
CREATE INDEX IF NOT EXISTS idx_messages_thread             ON messages(thread_id);
CREATE INDEX IF NOT EXISTS idx_messages_id_header          ON messages(message_id_header);
CREATE INDEX IF NOT EXISTS idx_memories_scope_type         ON memories(scope, type);
CREATE INDEX IF NOT EXISTS idx_memories_key                ON memories(key);
CREATE INDEX IF NOT EXISTS idx_tasks_status_due            ON tasks(status, due_at);
CREATE INDEX IF NOT EXISTS idx_pending_ops_status          ON pending_ops(status);

-- FTS5 virtual table for full-text search over messages
CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
    subject,
    body_text,
    from_name,
    content='messages',
    content_rowid='id'
);

-- Triggers to keep FTS index in sync with messages table
CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, subject, body_text, from_name)
    VALUES (new.id, new.subject, new.body_text, new.from_name);
END;

CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, subject, body_text, from_name)
    VALUES ('delete', old.id, old.subject, old.body_text, old.from_name);
END;

CREATE TRIGGER IF NOT EXISTS messages_au AFTER UPDATE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, subject, body_text, from_name)
    VALUES ('delete', old.id, old.subject, old.body_text, old.from_name);
    INSERT INTO messages_fts(rowid, subject, body_text, from_name)
    VALUES (new.id, new.subject, new.body_text, new.from_name);
END;
