export type ProviderKind =
  | "gmail"
  | "outlook"
  | "icloud"
  | "qq"
  | "mail163"
  | "generic_imap";

export interface Account {
  id: number;
  provider: ProviderKind;
  email: string;
  display_name?: string;
  status: "active" | "error" | "syncing";
}

export interface Folder {
  id: number;
  account_id: number;
  name: string;
  kind?: "inbox" | "sent" | "drafts" | "archive" | "spam" | "trash" | "custom";
  imap_path: string;
}

export interface EmailMessage {
  id: number;
  account_id: number;
  folder_id: number;
  uid: number;
  message_id_header?: string;
  thread_id?: string;
  from_addr?: string;
  from_name?: string;
  to_addrs?: string; // JSON array
  cc_addrs?: string;
  subject?: string;
  date?: number; // unix seconds
  snippet?: string;
  body_text?: string;
  body_html?: string;
  flags?: string; // JSON
  labels?: string; // JSON
  detected_lang?: string;
  has_attachments: number;
  size?: number;
  created_at: number;
}

export type AgentCategory =
  | "urgent"
  | "work"
  | "personal"
  | "newsletter"
  | "transactional"
  | "social"
  | "spam";

export interface SuggestedTask {
  title: string;
  due_at?: number;
}

export interface MessageEnrichment {
  message_id: number;
  category?: AgentCategory;
  priority?: number; // 0-100
  summary?: string; // already translated to display_lang
  facts?: string[];
  is_actionable?: boolean;
  needs_response?: boolean;
  task_count?: number;
  tasks?: SuggestedTask[];
}

export interface InboxItem {
  message: EmailMessage;
  enrichment?: MessageEnrichment; // may be undefined if not processed yet
}
