import { useEffect, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { RefreshCw, Search, ArrowLeft, Sparkles, Menu } from "lucide-react";
import { cn } from "@/lib/utils";
import { call, on, events } from "@/lib/tauri";
import { useInboxStore } from "@/stores/inboxStore";
import { useScreenSize } from "@/hooks/useMediaQuery";
import { InboxList } from "@/components/email/InboxList";
import { MessageDetail } from "@/components/email/MessageDetail";
import type { InboxFilter } from "@/stores/inboxStore";
import type { InboxItem } from "@/types/email";

type FilterId = "all" | "today" | "week" | "newsletters" | "done";

const FILTER_CHIPS: Array<{ id: FilterId; labelKey: string }> = [
  { id: "all", labelKey: "inbox.filters.all" },
  { id: "today", labelKey: "inbox.filters.today" },
  { id: "week", labelKey: "inbox.filters.week" },
  { id: "newsletters", labelKey: "inbox.filters.newsletters" },
  { id: "done", labelKey: "inbox.filters.done" },
];

interface InboxProps {
  onOpenSidebar?: () => void;
}

export function Inbox({ onOpenSidebar }: InboxProps = {}) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [searchParams, setSearchParams] = useSearchParams();
  const { selectedMessageId, filter, setFilter, setSelected } = useInboxStore();
  const { isMobile } = useScreenSize();

  const [search, setSearch] = useState("");
  const [isSyncing, setIsSyncing] = useState(false);

  // Sync URL → store filter
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

  // Subscribe to backend events for invalidation.
  useEffect(() => {
    let unlistenNew: (() => void) | undefined;
    let unlistenProcessed: (() => void) | undefined;
    let unlistenUpdated: (() => void) | undefined;

    on(events.EmailNew, () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    }).then((fn) => {
      unlistenNew = fn;
    });
    on(events.EmailProcessed, () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    }).then((fn) => {
      unlistenProcessed = fn;
    });
    on("message:updated", () => {
      queryClient.invalidateQueries({ queryKey: ["messages"] });
    }).then((fn) => {
      unlistenUpdated = fn;
    });

    return () => {
      unlistenNew?.();
      unlistenProcessed?.();
      unlistenUpdated?.();
    };
  }, [queryClient]);

  // Find selected item from current list cache.
  const { data: items = [] } = useQuery<InboxItem[]>({
    queryKey: ["messages", filter],
    enabled: false, // populated by InboxList; we just read the cache
  });
  const selectedItem = selectedMessageId
    ? items.find((i) => i.message.id === selectedMessageId) ?? null
    : null;

  async function handleRefresh() {
    setIsSyncing(true);
    try {
      await call("trigger_sync", {});
    } catch (err) {
      console.warn("trigger_sync failed", err);
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

  const hasDetail = selectedItem !== null;
  const showListOnly = isMobile ? !hasDetail : true;
  const showDetailOnly = isMobile && hasDetail;

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* Toolbar */}
      <div className="toolbar flex items-center gap-2 px-3 py-2 md:px-4 md:py-2.5">
        {/* Hamburger to open Sidebar on mobile */}
        {isMobile && !showDetailOnly && onOpenSidebar && (
          <button
            type="button"
            onClick={onOpenSidebar}
            aria-label={t("sidebar.toggle")}
            className="flex h-9 w-9 items-center justify-center rounded-md text-foreground hover:bg-accent"
          >
            <Menu className="h-5 w-5" />
          </button>
        )}

        {/* Back button on mobile when viewing a message */}
        {showDetailOnly && (
          <button
            type="button"
            onClick={() => setSelected(null)}
            aria-label={t("inbox.back_to_list")}
            className="flex h-9 items-center gap-1 rounded-md px-2 text-primary hover:bg-accent"
          >
            <ArrowLeft className="h-5 w-5" />
            <span className="text-[15px] font-medium">
              {t("inbox.back_to_list")}
            </span>
          </button>
        )}

        {/* Filter chips — hidden when viewing detail on mobile */}
        {!showDetailOnly && (
          <div className="flex flex-1 items-center gap-1 overflow-x-auto">
            {FILTER_CHIPS.map(({ id, labelKey }) => (
              <button
                key={id}
                type="button"
                onClick={() => handleFilterChip(id)}
                className={cn(
                  "shrink-0 rounded-full px-3 py-1 text-xs font-medium transition-colors no-select",
                  activeFilterId === id
                    ? "bg-primary text-primary-foreground"
                    : "bg-muted text-muted-foreground hover:bg-accent hover:text-foreground"
                )}
              >
                {t(labelKey)}
              </button>
            ))}
          </div>
        )}

        {/* Search — hidden on phones */}
        {!isMobile && !showDetailOnly && (
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground pointer-events-none" />
            <input
              type="search"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={t("inbox.search_placeholder")}
              className={cn(
                "h-8 w-40 rounded-md border border-border bg-muted/40 pl-8 pr-3 text-xs text-foreground placeholder:text-muted-foreground lg:w-56",
                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
              )}
            />
          </div>
        )}

        {!showDetailOnly && (
          <button
            type="button"
            onClick={handleRefresh}
            disabled={isSyncing}
            title={t("inbox.refresh")}
            aria-label={t("inbox.refresh")}
            className={cn(
              "flex h-9 w-9 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground",
              "disabled:opacity-50"
            )}
          >
            <RefreshCw className={cn("h-4 w-4", isSyncing && "animate-spin")} />
          </button>
        )}
      </div>

      {/* Content */}
      <div className="flex min-h-0 flex-1">
        {showListOnly && (
          <div
            className={cn(
              "flex min-w-0 flex-col border-r border-border",
              hasDetail && !isMobile ? "w-[440px] shrink-0 lg:w-[480px]" : "flex-1"
            )}
          >
            <InboxList search={search} />
          </div>
        )}

        {hasDetail && (
          <div
            className={cn(
              "min-w-0 overflow-hidden",
              isMobile ? "flex-1" : "flex-1"
            )}
          >
            <MessageDetail item={selectedItem!} />
          </div>
        )}

        {/* Empty pane on desktop when no message selected */}
        {!hasDetail && !isMobile && (
          <div className="hidden min-w-0 flex-1 items-center justify-center bg-muted/20 md:flex">
            <div className="flex flex-col items-center gap-2 text-muted-foreground">
              <Sparkles className="h-8 w-8 opacity-40" />
              <span className="text-sm">{t("inbox.no_selection")}</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
