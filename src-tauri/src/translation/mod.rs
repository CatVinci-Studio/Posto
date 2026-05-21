// Translation module.
//
// Provides cached LLM-based translation for email bodies.
// Translations are stored in the `translations` table and served from cache
// on subsequent requests for the same (message_id, target_lang) pair.

pub mod commands;

pub use commands::translate_message;
