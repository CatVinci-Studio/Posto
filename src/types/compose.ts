export interface DraftForm {
  from_account_id?: number;
  to: string[];
  cc: string[];
  bcc: string[];
  subject: string;
  body: string;
  reply_to_message_id?: number;
}

export interface SendResult {
  success: boolean;
  message_id?: number;
  error?: string;
}
