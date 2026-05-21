import * as React from "react";
import { call, on } from "@/lib/tauri";
import type { AgentRunSummary } from "@/types/activity";

// ---------------------------------------------------------------------------
// Mock data
// ---------------------------------------------------------------------------
const now = Math.floor(Date.now() / 1000);
const mins = (n: number) => now - n * 60;
const hrs = (n: number) => now - n * 3600;

const MOCK_RUNS: AgentRunSummary[] = [
  {
    id: 1,
    message_id: 1,
    agent_type: "triage",
    input_summary: "Triage: marked as Urgent",
    output_summary: "Priority set to 92",
    reversible: true,
    reverted: false,
    model: "gpt-4o-mini",
    tokens_in: 320,
    tokens_out: 45,
    created_at: mins(4),
    related_subject: "Q3 board deck — please review before Thursday",
  },
  {
    id: 2,
    message_id: 7,
    agent_type: "triage",
    input_summary: "Triage: marked as Newsletter",
    output_summary: "Moved to newsletter filter",
    reversible: true,
    reverted: false,
    model: "gpt-4o-mini",
    tokens_in: 210,
    tokens_out: 30,
    created_at: mins(12),
    related_subject: "TLDR 2026-05-21: Apple's new AI chip, Rust 2.0 ships",
  },
  {
    id: 3,
    message_id: 1,
    agent_type: "summary",
    input_summary: "Summary generated",
    output_summary: "CEO needs deck review before Thursday board meeting",
    reversible: false,
    reverted: false,
    model: "gpt-4o-mini",
    tokens_in: 480,
    tokens_out: 62,
    created_at: mins(18),
    related_subject: "Q3 board deck — please review before Thursday",
  },
  {
    id: 4,
    message_id: 4,
    agent_type: "draft",
    input_summary: "Draft reply generated",
    output_summary: "Reply draft ready for review",
    reversible: true,
    reverted: false,
    model: "gpt-4o-mini",
    tokens_in: 540,
    tokens_out: 180,
    created_at: hrs(1),
    related_subject: "Hiking trip this weekend?",
  },
  {
    id: 5,
    message_id: 13,
    agent_type: "rule",
    input_summary: "Rule applied: auto-archive spam",
    output_summary: "Moved to Spam folder",
    reversible: true,
    reverted: false,
    model: undefined,
    created_at: hrs(2),
    related_subject: "Flash sale: 50% off everything today only!!!",
  },
  {
    id: 6,
    message_id: 8,
    agent_type: "action",
    input_summary: "Action: converted to 3 tasks",
    output_summary: "Tasks RET-88, RET-89, RET-90 created",
    reversible: true,
    reverted: true,
    model: "gpt-4o-mini",
    tokens_in: 400,
    tokens_out: 90,
    created_at: hrs(3),
    related_subject: "Jordan Kim assigned you 3 issues",
  },
  {
    id: 7,
    message_id: 3,
    agent_type: "summary",
    input_summary: "Summary generated",
    output_summary: "PR #42 opened with 12 files changed",
    reversible: false,
    reverted: false,
    model: "gpt-4o-mini",
    tokens_in: 350,
    tokens_out: 55,
    created_at: hrs(4),
    related_subject: "[retposto/retposto] PR #42: feat: inbox view scaffold",
  },
  {
    id: 8,
    message_id: undefined,
    agent_type: "reflection",
    input_summary: "Daily reflection: categorization patterns reviewed",
    output_summary: "Memory updated with 3 new user preferences",
    reversible: false,
    reverted: false,
    model: "gpt-4o-mini",
    tokens_in: 1200,
    tokens_out: 200,
    created_at: hrs(6),
    related_subject: undefined,
  },
];

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------
interface UseAgentActivityOptions {
  paused: boolean;
}

interface UseAgentActivityResult {
  items: AgentRunSummary[];
  refetch: () => void;
  paused: boolean;
  setPaused: (v: boolean) => void;
}

export function useAgentActivity(opts: UseAgentActivityOptions): UseAgentActivityResult {
  const [items, setItems] = React.useState<AgentRunSummary[]>([]);
  const [paused, setPaused] = React.useState(opts.paused);

  // Initial fetch
  const fetchItems = React.useCallback(async () => {
    try {
      const data = await call<AgentRunSummary[]>("list_agent_runs", { limit: 50 });
      setItems(data);
    } catch {
      // Mock fallback
      setItems(MOCK_RUNS);
    }
  }, []);

  React.useEffect(() => {
    fetchItems();
  }, [fetchItems]);

  // Real-time subscription
  React.useEffect(() => {
    let unlisten: (() => void) | undefined;

    on<AgentRunSummary>("agent:run", (payload) => {
      if (!paused) {
        setItems((prev) => [payload, ...prev]);
      }
    })
      .then((fn) => {
        unlisten = fn;
      })
      .catch(() => {
        // Tauri not available in dev
      });

    return () => unlisten?.();
  }, [paused]);

  return {
    items,
    refetch: fetchItems,
    paused,
    setPaused,
  };
}
