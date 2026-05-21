export type AgentType =
  | "triage"
  | "summary"
  | "action"
  | "draft"
  | "rule"
  | "reflection";

export interface AgentRunSummary {
  id: number;
  message_id?: number;
  agent_type: AgentType;
  /** Short headline describing what was done */
  input_summary: string;
  /** Optional one-line result */
  output_summary?: string;
  reversible: boolean;
  reverted: boolean;
  model?: string;
  tokens_in?: number;
  tokens_out?: number;
  /** Unix seconds */
  created_at: number;
  /** Email subject if applicable */
  related_subject?: string;
}
