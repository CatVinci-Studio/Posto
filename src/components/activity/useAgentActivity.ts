import * as React from "react";
import { call, on } from "@/lib/tauri";
import type { AgentRunSummary } from "@/types/activity";

interface UseAgentActivityOptions {
  paused: boolean;
}

interface UseAgentActivityResult {
  items: AgentRunSummary[];
  refetch: () => void;
  paused: boolean;
  setPaused: (v: boolean) => void;
  error: string | null;
}

export function useAgentActivity(
  opts: UseAgentActivityOptions
): UseAgentActivityResult {
  const [items, setItems] = React.useState<AgentRunSummary[]>([]);
  const [paused, setPaused] = React.useState(opts.paused);
  const [error, setError] = React.useState<string | null>(null);

  const fetchItems = React.useCallback(async () => {
    try {
      const data = await call<AgentRunSummary[]>("list_agent_runs", {
        message_id: null,
        limit: 50,
      });
      setItems(data);
      setError(null);
    } catch (err) {
      setItems([]);
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  React.useEffect(() => {
    fetchItems();
  }, [fetchItems]);

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
        // Tauri event channel unavailable (e.g. headless / web preview)
      });

    return () => unlisten?.();
  }, [paused]);

  return {
    items,
    refetch: fetchItems,
    paused,
    setPaused,
    error,
  };
}
