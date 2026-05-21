//! Message-action IPC commands (read / flagged / archive / delete) plus an
//! enriched inbox query that joins messages with their latest agent run.
//!
//! V1 semantics are local-only: state lives in `messages.flags` (JSON array of
//! IMAP-style strings) and IMAP-side mirroring is queued in `pending_ops` for
//! a later flush worker. This keeps the UI snappy while preserving the
//! intent for an eventual round-trip to the server.

pub mod commands;
