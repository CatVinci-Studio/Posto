import { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import { RefreshCw, Search, Sparkles } from "lucide-react";
import { cn } from "@/lib/utils";
import { call, on, events } from "@/lib/tauri";
import { useInboxStore } from "@/stores/inboxStore";
import { InboxList } from "@/components/email/InboxList";
import { MessageDetail } from "@/components/email/MessageDetail";
import { getMockItems } from "@/lib/mockData";
import type { InboxFilter } from "@/stores/inboxStore";

// Filter chip definition
type FilterId = "all" | "today" | "week" | "newsletters" | "done";

const FILTER_CHIPS: Array<{ id: FilterId; labelKey: string }> = [
  { id: "all",         labelKey: "inbox.filters.all" },
  { id: "today",       labelKey: "inbox.filters.today" },
  { id: "week",        labelKey: "inbox.filters.week" },
  { id: "newsletters", labelKey: "inbox.filters.newsletters" },
  { id: "done",        labelKey: "inbox.filters.done" },
];

// Agent Activity Drawer (right panel at >1400px)
function ActivityDrawer() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-3 p-4">
      <div className="flex items-center gap-2">
        <Sparkles className="h-4 w-4 text-primary" />
        <span className="text-sm font-semibold">{t("sidebar.agent_activity")}</span>
      </div>
      <p className="text-xs text-muted-foreground">
        {/* TODO: Subscribe to agent:run events and display real activity */}
        No recent agent activity.
      </p>
    </div>
  );
}

export function Inbox() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const { selectedMessageId, filter, setFilter, showActivityDrawer } = useInboxStore();

  const [search, setSearch] = useState("");
  const [isSyncing, setIsSyncing] = useState(false);

  // Sync URL search params → store filter on mount / when URL changes
  useEffect(() => {
    const urlFilter = searchParams.get("filter") as FilterId | null;
    const urlAccount = searchParams.get("account");

    const newFilter: InboxFilter = {};
    if (urlFilter && urlFilter !== "all") {
      newFilter.status = urlFilter as InboxFilter["status"];
    }
    if (urlAccount) {
      newFilter.account_id = Number(urlAccount);
    }
    setFilter(newFilter);
  }, [searchParams, setFilter]);

  // Subscribe to backend events and refetch
  useEffect(() => {
    let unlistenNew: (() => void) | undefined;
    let unlistenProcessed: (() => void) | undefined;

    on(events.EmailNew, () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    }).then((fn) => { unlistenNew = fn; });

    on(events.EmailProcessed, () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    }).then((fn) => { unlistenProcessed = fn; });

    return () => {
      unlistenNew?.();
      unlistenProcessed?.();
    };
  }, [queryClient]);

  async function handleRefresh() {
    setIsSyncing(true);
    try {
      await call("trigger_sync", {});
    } catch {
      // Backend not ready — still invalidate cache to re-run query with mock data
    }
    queryClient.invalidateQueries({ queryKey: ["messages"] });
    setTimeout(() => setIsSyncing(false), 600);
  }

  function handleFilterChip(id: FilterId) {
    const params = new URLSearchParams(searchParams);
    if (id === "all") {
      params.delete("filter");
    } else {
      params.set("filter", id);
    }
    setSearchParams(params);
  }

  const activeFilterId: FilterId = (filter.status as FilterId) ?? "all";

  // Find selected item from mock (TODO: look up from react-query cache when real)
  const allItems = getMockItems();
  const selectedItem = selectedMessageId
    ? allItems.find((i) => i.message.id === selectedMessageId) ?? null
    : null;

  const hasDetail = selectedItem !== null;

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center gap-2 border-b border-border bg-background px-4 py-2.5">
        {/* Filter chips */}
        <div className="flex items-center gap-1">
          {FILTER_CHIPS.map(({ id, labelKey }) => (
            <button
              key={id}
              type="button"
              onClick={() => handleFilterChip(id)}
              className={cn(
                "rounded-full px-3 py-1 text-xs font-medium transition-colors",
                activeFilterId === id
                  ? "bg-primary text-primary-foreground"
                  : "bg-muted text-muted-foreground hover:bg-accent hover:text-foreground"
              )}
            >
              {t(labelKey)}
            </button>
          ))}
        </div>

        <div className="flex-1" />

        {/* Search */}
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground pointer-events-none" />
          <input
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t("inbox.search_placeholder")}
            className={cn(
              "h-8 w-48 rounded-md border border-border bg-muted/40 pl-8 pr-3 text-xs text-foreground placeholder:text-muted-foreground",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
            )}
          />
        </div>

        {/* Refresh */}
        <button
          type="button"
          onClick={handleRefresh}
          disabled={isSyncing}
          title={t("inbox.refresh")}
          aria-label={t("inbox.refresh")}
          className={cn(
            "flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground",
            "disabled:opacity-50"
          )}
        >
          <RefreshCw className={cn("h-4 w-4", isSyncing && "animate-spin")} />
        </button>
      </div>

      {/* Content area */}
      <div className="flex min-h-0 flex-1">
        {/* Email list */}
        <div
          className={cn(
            "flex flex-col border-r border-border",
            hasDetail ? "w-[480px] shrink-0" : "flex-1"
          )}
        >
          <InboxList search={search} />
        </div>

        {/* Detail pane */}
        {hasDetail && (
          <div className="min-w-0 flex-1 overflow-hidden">
            <MessageDetail item={selectedItem} />
          </div>
        )}

        {/* Agent activity drawer — only at >1400px when open */}
        {showActivityDrawer && hasDetail && (
          <div className="hidden w-64 shrink-0 border-l border-border bg-muted/20 xl:block 2xl:block">
            <ActivityDrawer />
          </div>
        )}
      </div>
    </div>
  );
}
