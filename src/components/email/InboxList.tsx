import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { call } from "@/lib/tauri";
import { getMockItems } from "@/lib/mockData";
import { useInboxStore } from "@/stores/inboxStore";
import { EmptyState } from "./EmptyState";
import { EmailCard } from "./EmailCard";
import type { InboxItem } from "@/types/email";
import type { InboxFilter } from "@/stores/inboxStore";

// ---- Date bucketing helpers ----
const DAY_S = 86400;

function isToday(dateUnix: number): boolean {
  const now = Math.floor(Date.now() / 1000);
  const sod = now - (now % DAY_S); // start of day UTC
  return dateUnix >= sod;
}

function isYesterday(dateUnix: number): boolean {
  const now = Math.floor(Date.now() / 1000);
  const sod = now - (now % DAY_S);
  return dateUnix >= sod - DAY_S && dateUnix < sod;
}

function isThisWeek(dateUnix: number): boolean {
  const now = Math.floor(Date.now() / 1000);
  const sod = now - (now % DAY_S);
  return dateUnix >= sod - 6 * DAY_S && dateUnix < sod - DAY_S;
}

type SectionKey = "today" | "yesterday" | "this_week" | "older";

function getSectionKey(dateUnix?: number): SectionKey {
  if (!dateUnix) return "older";
  if (isToday(dateUnix)) return "today";
  if (isYesterday(dateUnix)) return "yesterday";
  if (isThisWeek(dateUnix)) return "this_week";
  return "older";
}

// ---- Data fetching with mock fallback ----
async function fetchMessages(filter: InboxFilter): Promise<InboxItem[]> {
  try {
    const items = await call<InboxItem[]>("list_messages", {
      account_id: filter.account_id ?? null,
      category: filter.category ?? null,
      status: filter.status ?? null,
    });
    return items;
  } catch {
    // Backend not ready — use mock data
    return getMockItems();
  }
}

function filterItems(items: InboxItem[], filter: InboxFilter, search: string): InboxItem[] {
  let result = items;

  if (filter.status === "newsletters") {
    result = result.filter((i) => i.enrichment?.category === "newsletter");
  } else if (filter.status === "today") {
    result = result.filter((i) => isToday(i.message.date ?? 0));
  } else if (filter.status === "week") {
    const now = Math.floor(Date.now() / 1000);
    result = result.filter(
      (i) => (i.message.date ?? 0) >= now - 7 * DAY_S
    );
  } else if (filter.status === "done") {
    result = result.filter((i) => {
      const flags = i.message.flags ? JSON.parse(i.message.flags) : [];
      return Array.isArray(flags) && flags.includes("\\Seen");
    });
  }

  if (filter.category) {
    result = result.filter((i) => i.enrichment?.category === filter.category);
  }

  if (filter.account_id != null) {
    result = result.filter((i) => i.message.account_id === filter.account_id);
  }

  if (search.trim()) {
    const q = search.toLowerCase();
    result = result.filter(
      (i) =>
        i.message.subject?.toLowerCase().includes(q) ||
        i.message.from_name?.toLowerCase().includes(q) ||
        i.message.from_addr?.toLowerCase().includes(q) ||
        i.enrichment?.summary?.toLowerCase().includes(q)
    );
  }

  return result;
}

// ---- Component ----

interface InboxListProps {
  search: string;
}

export function InboxList({ search }: InboxListProps) {
  const { t } = useTranslation();
  const { filter } = useInboxStore();

  const { data: items = [], isLoading } = useQuery({
    queryKey: ["messages", filter],
    queryFn: () => fetchMessages(filter),
    staleTime: 30_000,
  });

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <span className="text-sm text-muted-foreground animate-pulse">
          Loading…
        </span>
      </div>
    );
  }

  const filtered = filterItems(items, filter, search);

  if (filtered.length === 0) {
    const emptyTitle =
      filter.status === "newsletters"
        ? t("inbox.empty_newsletters")
        : filter.status === "done"
        ? t("inbox.empty_done")
        : t("inbox.empty.title");

    return (
      <EmptyState
        icon="📬"
        title={emptyTitle}
        subtitle={t("inbox.empty.subtitle")}
      />
    );
  }

  // Group by date section
  const SECTION_ORDER: SectionKey[] = ["today", "yesterday", "this_week", "older"];
  const sections = new Map<SectionKey, InboxItem[]>();
  for (const item of filtered) {
    const key = getSectionKey(item.message.date);
    if (!sections.has(key)) sections.set(key, []);
    sections.get(key)!.push(item);
  }

  return (
    <div className="overflow-y-auto h-full">
      {SECTION_ORDER.filter((k) => sections.has(k)).map((sectionKey) => (
        <div key={sectionKey}>
          <div className="sticky top-0 z-10 border-b border-border bg-muted/60 px-3 py-1.5 backdrop-blur-sm">
            <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {t(`inbox.sections.${sectionKey}`)}
            </span>
          </div>
          {sections.get(sectionKey)!.map((item) => (
            <EmailCard key={item.message.id} item={item} />
          ))}
        </div>
      ))}
    </div>
  );
}
