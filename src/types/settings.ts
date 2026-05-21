export type ProviderKind = "gmail" | "outlook" | "icloud" | "qq" | "mail163" | "generic_imap";
export type UiLang = "zh" | "en";
export type DisplayLang = "zh" | "en" | "ja" | "original";
export type ReplyLang = "auto" | "zh" | "en" | "ja";
export type TrustLevel = "aggressive" | "medium" | "conservative";
export type MemoryType = "contact" | "preference" | "project" | "rule" | "fact";

export interface AppSettings {
  ui_lang: UiLang;
  display_lang: DisplayLang;
  reply_lang: ReplyLang;
  trust_level: TrustLevel;
  theme: "system" | "light" | "dark";
  llm_provider: "openai";
  llm_model: string;
  per_action: {
    classify: "auto" | "ask";
    label: "auto" | "ask";
    archive_newsletter: "auto" | "ask";
    move_folder: "auto" | "ask";
    mark_read: "auto" | "ask";
    generate_draft: "auto" | "ask";
  };
  // send / delete are always "ask"
}

export interface AccountSummary {
  id: number;
  provider: ProviderKind;
  email: string;
  display_name?: string;
  status: "active" | "error" | "syncing";
  last_sync_at?: number;
  error_message?: string;
}

export interface MemoryEntry {
  id: number;
  type: MemoryType;
  scope: string; // "global" | "account:N" | "contact:email"
  key?: string;
  content: string;
  importance: number; // 0-1
  pinned: 0 | 1;
  source_message_id?: number;
  created_at: number;
  last_used_at?: number;
  use_count: number;
}
