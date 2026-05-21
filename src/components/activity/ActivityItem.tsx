import * as React from "react";
import { useTranslation } from "react-i18next";
import {
  Filter,
  FileText,
  CheckSquare,
  Zap,
  PenTool,
  Brain,
  RotateCcw,
} from "lucide-react";
import { cn, formatRelativeTime } from "@/lib/utils";
import { call } from "@/lib/tauri";
import { Button } from "@/components/ui/Button";
import type { AgentRunSummary, AgentType } from "@/types/activity";

// ---------------------------------------------------------------------------
// Icon map
// ---------------------------------------------------------------------------
function AgentIcon({ type, className }: { type: AgentType; className?: string }) {
  const cls = cn("h-3.5 w-3.5", className);
  switch (type) {
    case "triage":
      return <Filter className={cls} />;
    case "summary":
      return <FileText className={cls} />;
    case "action":
      return <CheckSquare className={cls} />;
    case "rule":
      return <Zap className={cls} />;
    case "draft":
      return <PenTool className={cls} />;
    case "reflection":
      return <Brain className={cls} />;
    default:
      return <CheckSquare className={cls} />;
  }
}

// Accent colors per agent type
const TYPE_COLORS: Record<AgentType, string> = {
  triage: "bg-blue-500/10 text-blue-600 dark:text-blue-400",
  summary: "bg-violet-500/10 text-violet-600 dark:text-violet-400",
  action: "bg-green-500/10 text-green-600 dark:text-green-400",
  rule: "bg-amber-500/10 text-amber-600 dark:text-amber-400",
  draft: "bg-pink-500/10 text-pink-600 dark:text-pink-400",
  reflection: "bg-sky-500/10 text-sky-600 dark:text-sky-400",
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------
interface ActivityItemProps {
  run: AgentRunSummary;
  onReverted?: (id: number) => void;
}

export function ActivityItem({ run, onReverted }: ActivityItemProps) {
  const { t, i18n } = useTranslation();
  const [undoing, setUndoing] = React.useState(false);
  const [reverted, setReverted] = React.useState(run.reverted);

  const relativeTime = formatRelativeTime(
    run.created_at * 1000,
    i18n.language === "zh" ? "zh-CN" : "en-US"
  );

  const handleUndo = async () => {
    setUndoing(true);
    try {
      try {
        await call("undo_agent_action", { agent_run_id: run.id });
      } catch {
        // Mock: simulate success
        await new Promise((r) => setTimeout(r, 600));
      }
      setReverted(true);
      onReverted?.(run.id);
    } finally {
      setUndoing(false);
    }
  };

  return (
    <div
      className={cn(
        "group relative flex gap-3 rounded-lg p-3 transition-colors hover:bg-accent/50",
        reverted && "opacity-60"
      )}
    >
      {/* Icon badge */}
      <div
        className={cn(
          "mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-full",
          TYPE_COLORS[run.agent_type]
        )}
      >
        <AgentIcon type={run.agent_type} />
      </div>

      {/* Content */}
      <div className="min-w-0 flex-1">
        <p
          className={cn(
            "text-sm font-medium text-foreground leading-snug",
            reverted && "line-through text-muted-foreground"
          )}
        >
          {run.input_summary}
        </p>

        {run.related_subject && (
          <p className="mt-0.5 truncate text-xs text-muted-foreground">
            {run.related_subject}
          </p>
        )}

        {run.output_summary && !reverted && (
          <p className="mt-0.5 text-xs text-muted-foreground/80">
            {run.output_summary}
          </p>
        )}

        <div className="mt-1 flex items-center gap-2">
          <span className="text-xs text-muted-foreground">{relativeTime}</span>

          {/* Affected message link */}
          {run.message_id && (
            <a
              href={`#message/${run.message_id}`}
              className="text-xs text-primary underline-offset-2 hover:underline focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring rounded"
            >
              #{run.message_id}
            </a>
          )}

          {/* Undo */}
          {run.reversible && !reverted && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={handleUndo}
              disabled={undoing}
              className="h-5 px-1.5 py-0 text-xs text-muted-foreground hover:text-foreground"
            >
              {undoing ? (
                <RotateCcw className="h-3 w-3 animate-spin" />
              ) : (
                t("activity.actions.undo")
              )}
            </Button>
          )}

          {reverted && (
            <span className="text-xs text-muted-foreground italic">
              {t("activity.actions.undone")}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
