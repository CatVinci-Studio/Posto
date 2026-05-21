import { useTranslation } from "react-i18next";
import { useQueryClient } from "@tanstack/react-query";
import { Archive, Reply, ListTodo, AlertCircle, Star, Trash2, Paperclip } from "lucide-react";
import { cn, formatRelativeTime } from "@/lib/utils";
import { call } from "@/lib/tauri";
import { useInboxStore } from "@/stores/inboxStore";
import { Swipeable } from "@/components/ui/Swipeable";
import { Avatar } from "./Avatar";
import { CategoryChip } from "./CategoryChip";
import type { InboxItem } from "@/types/email";

interface EmailCardProps {
  item: InboxItem;
  /** Render swipe-to-reveal actions on touch / narrow screens. */
  swipeEnabled?: boolean;
}

function parseFlags(raw?: string | null): string[] {
  if (!raw) return [];
  try {
    const v = JSON.parse(raw);
    return Array.isArray(v) ? v : [];
  } catch {
    return [];
  }
}

export function EmailCard({ item, swipeEnabled = false }: EmailCardProps) {
  const { t, i18n } = useTranslation();
  const queryClient = useQueryClient();
  const { selectedMessageId, setSelected } = useInboxStore();
  const { message, enrichment } = item;

  const flags = parseFlags(message.flags);
  const isUnread = !flags.includes("\\Seen");
  const isFlagged = flags.includes("\\Flagged");

  const isSelected = selectedMessageId === message.id;
  const priority = enrichment?.priority ?? 0;
  const hasPriorityDot = priority >= 40;
  const priorityColor =
    priority >= 70 ? "bg-red-500" : "bg-amber-400";

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["messages"] });
  }

  async function handleArchive(e?: React.MouseEvent) {
    e?.stopPropagation();
    await call("archive_message", { message_id: message.id }).catch(() => null);
    invalidate();
    if (isSelected) setSelected(null);
  }

  async function handleDelete(e?: React.MouseEvent) {
    e?.stopPropagation();
    await call("delete_message", { message_id: message.id }).catch(() => null);
    invalidate();
    if (isSelected) setSelected(null);
  }

  async function handleFlag(e?: React.MouseEvent) {
    e?.stopPropagation();
    await call("flag_message", {
      message_id: message.id,
      flagged: !isFlagged,
    }).catch(() => null);
    invalidate();
  }

  async function handleReply(e?: React.MouseEvent) {
    e?.stopPropagation();
    setSelected(message.id);
    // Reply navigation handled by MessageDetail toolbar.
  }

  function handleClick() {
    if (!isSelected && isUnread) {
      // Mark read once user opens it.
      call("mark_read", { message_id: message.id, read: true }).catch(() => null);
    }
    setSelected(isSelected ? null : message.id);
  }

  const timeStr = message.date
    ? formatRelativeTime(
        message.date * 1000,
        i18n.language === "zh" ? "zh-CN" : "en-US"
      )
    : "";

  const card = (
    <div
      role="button"
      tabIndex={0}
      aria-pressed={isSelected}
      onClick={handleClick}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleClick();
        }
      }}
      className={cn(
        "group relative flex cursor-pointer items-start gap-3 border-b border-border px-3 py-3 transition-colors no-select",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset",
        isSelected
          ? "border-l-[3px] border-l-primary bg-accent/50 pl-[9px]"
          : "hover:bg-accent/30"
      )}
    >
      {/* Apple-style left rail: unread blue pip, otherwise blank space */}
      <div className="flex w-3 shrink-0 items-center justify-center pt-1">
        {isUnread ? (
          <span className="unread-pip" aria-label={t("inbox.unread")} />
        ) : null}
      </div>

      <Avatar
        name={message.from_name}
        email={message.from_addr}
        size="md"
        className="mt-0.5"
      />

      <div className="min-w-0 flex-1">
        <div className="flex items-baseline justify-between gap-2">
          <span
            className={cn(
              "truncate text-[15px] text-foreground",
              isUnread ? "font-semibold" : "font-medium"
            )}
          >
            {message.from_name ?? message.from_addr ?? t("inbox.unknown_sender")}
          </span>
          <span className="shrink-0 text-xs text-muted-foreground">
            {timeStr}
          </span>
        </div>

        <div className="mt-0.5 truncate text-[14px]">
          <span
            className={cn(
              "text-foreground",
              isUnread ? "font-medium" : ""
            )}
          >
            {message.subject ?? t("inbox.no_subject")}
          </span>
        </div>

        {(enrichment?.summary || message.snippet) && (
          <p className="mt-0.5 line-clamp-2 text-[13px] text-muted-foreground">
            {enrichment?.summary ?? message.snippet}
          </p>
        )}

        <div className="mt-1.5 flex items-center gap-2">
          {enrichment?.category && <CategoryChip category={enrichment.category as any} />}
          {hasPriorityDot && (
            <span
              className={cn("inline-block h-2 w-2 rounded-full", priorityColor)}
              title={`Priority: ${priority}`}
              aria-label={`Priority ${priority}`}
            />
          )}
          {enrichment?.needs_response && (
            <span className="flex items-center gap-0.5 text-xs text-amber-600 dark:text-amber-400">
              <AlertCircle className="h-3 w-3" />
              {t("inbox.needs_response")}
            </span>
          )}
          {(enrichment?.task_count ?? 0) > 0 && (
            <span className="flex items-center gap-0.5 text-xs text-muted-foreground">
              <ListTodo className="h-3 w-3" />
              {enrichment?.task_count}
            </span>
          )}
          {(message.has_attachments ?? 0) > 0 && (
            <Paperclip className="h-3 w-3 text-muted-foreground" />
          )}
        </div>
      </div>

      {/* Flag star (always visible, dim when not flagged) */}
      <button
        type="button"
        aria-label={
          isFlagged ? t("inbox.actions.unflag") : t("inbox.actions.flag")
        }
        onClick={handleFlag}
        className={cn(
          "ml-1 mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-md transition-colors",
          "opacity-0 hover:bg-accent group-hover:opacity-100",
          isFlagged && "opacity-100"
        )}
      >
        <Star
          className={cn(
            "h-4 w-4",
            isFlagged
              ? "fill-[color:hsl(var(--pip-flagged))] text-[color:hsl(var(--pip-flagged))]"
              : "text-muted-foreground"
          )}
        />
      </button>

      {/* Hover actions (desktop pointer device) */}
      <div
        className={cn(
          "absolute right-12 top-1/2 -translate-y-1/2 hidden items-center gap-0.5 rounded-md border border-border bg-background p-1 shadow-sm",
          "opacity-0 transition-opacity group-hover:opacity-100 sm:group-hover:flex",
          isSelected && "sm:group-hover:hidden"
        )}
        onClick={(e) => e.stopPropagation()}
      >
        <IconBtn
          onClick={handleArchive}
          title={t("inbox.actions.archive")}
          icon={<Archive className="h-3.5 w-3.5" />}
        />
        <IconBtn
          onClick={handleReply}
          title={t("inbox.actions.reply")}
          icon={<Reply className="h-3.5 w-3.5" />}
        />
        <IconBtn
          onClick={handleDelete}
          title={t("inbox.actions.delete")}
          icon={<Trash2 className="h-3.5 w-3.5" />}
        />
      </div>
    </div>
  );

  if (!swipeEnabled) return card;

  return (
    <Swipeable
      leftActions={[
        {
          label: t("inbox.actions.archive"),
          icon: <Archive className="h-5 w-5" />,
          color: "bg-blue-500",
          onAction: () => handleArchive(),
        },
        {
          label: t("inbox.actions.delete"),
          icon: <Trash2 className="h-5 w-5" />,
          color: "bg-destructive",
          onAction: () => handleDelete(),
        },
      ]}
      rightActions={[
        {
          label: isFlagged ? t("inbox.actions.unflag") : t("inbox.actions.flag"),
          icon: <Star className="h-5 w-5" />,
          color: "bg-[color:hsl(var(--pip-flagged))]",
          onAction: () => handleFlag(),
        },
      ]}
    >
      {card}
    </Swipeable>
  );
}

interface IconBtnProps {
  onClick: (e: React.MouseEvent) => void;
  title: string;
  icon: React.ReactNode;
}

function IconBtn({ onClick, title, icon }: IconBtnProps) {
  return (
    <button
      type="button"
      title={title}
      aria-label={title}
      onClick={onClick}
      className="flex h-6 w-6 items-center justify-center rounded text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
    >
      {icon}
    </button>
  );
}
