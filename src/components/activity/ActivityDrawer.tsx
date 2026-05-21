import * as React from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import { X, Pause, Play, Activity } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/Button";
import { useAgentActivity } from "./useAgentActivity";
import { ActivityItem } from "./ActivityItem";

interface ActivityDrawerProps {
  open: boolean;
  onClose: () => void;
}

/**
 * Right-side fixed drawer showing real-time agent activity.
 * Slides in from the right; self-contained with its own data hook.
 */
export function ActivityDrawer({ open, onClose }: ActivityDrawerProps) {
  const { t } = useTranslation();
  const { items, paused, setPaused } = useAgentActivity({ paused: false });

  // Close on Escape
  React.useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [open, onClose]);

  return (
    <>
      {/* Backdrop (mobile-friendly, non-blocking on desktop) */}
      {open && (
        <div
          className="fixed inset-0 z-30 bg-black/20 md:hidden"
          aria-hidden="true"
          onClick={onClose}
        />
      )}

      {/* Drawer panel */}
      <aside
        className={cn(
          "fixed right-0 top-0 z-40 flex h-full w-[360px] flex-col border-l border-border bg-background shadow-xl",
          "transition-transform duration-200 ease-in-out",
          open ? "translate-x-0" : "translate-x-full"
        )}
        aria-label={t("activity.title")}
      >
        {/* Header */}
        <header className="flex items-center justify-between border-b border-border px-5 py-3.5">
          <div className="flex items-center gap-2">
            <Activity className="h-4 w-4 text-primary" />
            <h2 className="text-sm font-semibold text-foreground">{t("activity.title")}</h2>
          </div>

          <div className="flex items-center gap-1">
            {/* Pause / Resume */}
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => setPaused(!paused)}
              aria-label={paused ? t("activity.resume") : t("activity.pause")}
              className="h-8 w-8 p-0"
            >
              {paused ? (
                <Play className="h-3.5 w-3.5" />
              ) : (
                <Pause className="h-3.5 w-3.5" />
              )}
            </Button>

            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={onClose}
              aria-label="Close activity drawer"
              className="h-8 w-8 p-0"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        </header>

        {/* Pause indicator */}
        {paused && (
          <div className="flex items-center justify-center gap-1.5 bg-amber-500/10 px-5 py-1.5 text-xs text-amber-600 dark:text-amber-400">
            <Pause className="h-3 w-3" />
            {t("activity.pause")} — new events are buffered
          </div>
        )}

        {/* Body — scrollable timeline */}
        <div className="flex-1 overflow-y-auto px-2 py-2">
          {items.length === 0 ? (
            <div className="flex h-32 items-center justify-center text-sm text-muted-foreground">
              {t("activity.empty")}
            </div>
          ) : (
            <div className="flex flex-col gap-0.5">
              {items.map((run) => (
                <ActivityItem key={run.id} run={run} />
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <footer className="border-t border-border px-5 py-3">
          <Link
            to="/activity"
            className="text-xs text-primary underline-offset-2 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 rounded"
            onClick={onClose}
          >
            {t("activity.show_history")} →
          </Link>
        </footer>
      </aside>
    </>
  );
}
