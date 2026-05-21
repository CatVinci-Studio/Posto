import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { call } from "@/lib/tauri";
import { useInboxStore } from "@/stores/inboxStore";
import { useScreenSize } from "@/hooks/useMediaQuery";
import { EmptyState } from "./EmptyState";
import { EmailCard } from "./EmailCard";
import type { InboxItem } from "@/types/email";
import type { InboxFilter } from "@/stores/inboxStore";

const DAY_S = 86400;

function isToday(dateUnix: number): boolean {
  const now = Math.floor(Date.now() / 1000);
  const sod = now - (now % DAY_S);
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

async function fetchMessages(filter: InboxFilter): Promise<InboxItem[]> {
  return await call<InboxItem[]>("list_inbox_items", {
    filter: {
      account_id: filter.account_id ?? null,
      category: filter.category ?? null,
      status: filter.status ?? null,
    },
  });
}

function applySearch(items: InboxItem[], search: string): InboxItem[] {
  if (!search.trim()) return items;
  const q = search.toLowerCase();
  return items.filter(
    (i) =>
      i.message.subject?.toLowerCase().includes(q) ||
      i.message.from_name?.toLowerCase().includes(q) ||
      i.message.from_addr?.toLowerCase().includes(q) ||
      i.message.snippet?.toLowerCase().includes(q) ||
      i.enrichment?.summary?.toLowerCase().includes(q)
  );
}

interface InboxListProps {
  search: string;
}

export function InboxList({ search }: InboxListProps) {
  const { t } = useTranslation();
  const { filter } = useInboxStore();
  const { isMobile } = useScreenSize();

  const { data: items = [], isLoading, isError } = useQuery({
    queryKey: ["messages", filter],
    queryFn: () => fetchMessages(filter),
    staleTime: 30_000,
  });

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <span className="text-sm text-muted-foreground animate-pulse">
          {t("inbox.loading")}
        </span>
      </div>
    );
  }

  if (isError) {
    return (
      <EmptyState
        icon="⚠️"
        title={t("inbox.error.title")}
        subtitle={t("inbox.error.subtitle")}
      />
    );
  }

  const filtered = applySearch(items, search);

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

  const SECTION_ORDER: SectionKey[] = [
    "today",
    "yesterday",
    "this_week",
    "older",
  ];
  const sections = new Map<SectionKey, InboxItem[]>();
  for (const item of filtered) {
    const key = getSectionKey(item.message.date);
    if (!sections.has(key)) sections.set(key, []);
    sections.get(key)!.push(item);
  }

  return (
    <div className="overflow-y-auto h-full scroll-smooth-y">
      {SECTION_ORDER.filter((k) => sections.has(k)).map((sectionKey) => (
        <div key={sectionKey}>
          <div className="sticky top-0 z-10 border-b border-border bg-background/85 px-4 py-1.5 backdrop-blur-xl">
            <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
              {t(`inbox.sections.${sectionKey}`)}
            </span>
          </div>
          {sections.get(sectionKey)!.map((item) => (
            <EmailCard
              key={item.message.id}
              item={item}
              swipeEnabled={isMobile}
            />
          ))}
        </div>
      ))}
    </div>
  );
}
