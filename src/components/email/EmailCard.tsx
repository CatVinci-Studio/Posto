import { useTranslation } from "react-i18next";
import { Archive, Reply, Clock, ListTodo, AlertCircle } from "lucide-react";
import { cn, formatRelativeTime } from "@/lib/utils";
import { useInboxStore } from "@/stores/inboxStore";
import { Avatar } from "./Avatar";
import { CategoryChip } from "./CategoryChip";
import type { InboxItem } from "@/types/email";

interface EmailCardProps {
  item: InboxItem;
}

export function EmailCard({ item }: EmailCardProps) {
  const { t, i18n } = useTranslation();
  const { selectedMessageId, setSelected } = useInboxStore();
  const { message, enrichment } = item;

  const isSelected = selectedMessageId === message.id;
  const priority = enrichment?.priority ?? 0;
  const hasPriorityDot = priority >= 40;
  const priorityColor =
    priority >= 70 ? "bg-red-500" : "bg-amber-400";

  function handleAction(
    e: React.MouseEvent,
    action: "archive" | "reply" | "snooze" | "task"
  ) {
    e.stopPropagation();
    // TODO: wire to real backend commands
    console.log(`action: ${action} on message ${message.id}`);
  }

  const timeStr = message.date
    ? formatRelativeTime(message.date * 1000, i18n.language === "zh" ? "zh-CN" : "en-US")
    : "";

  return (
    <div
      role="button"
      tabIndex={0}
      aria-pressed={isSelected}
      onClick={() => setSelected(isSelected ? null : message.id)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          setSelected(isSelected ? null : message.id);
        }
      }}
      className={cn(
        "group relative flex cursor-pointer items-start gap-3 border-b border-border px-3 py-3 transition-colors",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset",
        isSelected
          ? "border-l-2 border-l-primary bg-accent/60"
          : "hover:bg-accent/40"
      )}
    >
      {/* Avatar */}
      <Avatar
        name={message.from_name}
        email={message.from_addr}
        size="md"
        className="mt-0.5"
      />

      {/* Main content */}
      <div className="min-w-0 flex-1">
        {/* Row 1: sender + time */}
        <div className="flex items-baseline justify-between gap-2">
          <span className="truncate text-sm font-medium text-foreground">
            {message.from_name ?? message.from_addr ?? t("inbox.unknown_sender")}
          </span>
          <span className="shrink-0 text-xs text-muted-foreground">{timeStr}</span>
        </div>

        {/* Row 2: subject + summary */}
        <div className="mt-0.5 truncate text-sm">
          <span className="font-medium text-foreground">
            {message.subject ?? "(no subject)"}
          </span>
          {enrichment?.summary && (
            <>
              <span className="mx-1 text-muted-foreground">—</span>
              <span className="text-muted-foreground">{enrichment.summary}</span>
            </>
          )}
        </div>

        {/* Row 3: category chip + priority dot + task count */}
        <div className="mt-1.5 flex items-center gap-2">
          {enrichment?.category && (
            <CategoryChip category={enrichment.category} />
          )}
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
          {enrichment?.task_count != null && enrichment.task_count > 0 && (
            <span className="flex items-center gap-0.5 text-xs text-muted-foreground">
              <ListTodo className="h-3 w-3" />
              {enrichment.task_count}
            </span>
          )}
          {message.has_attachments > 0 && (
            <span className="text-xs text-muted-foreground">📎</span>
          )}
        </div>
      </div>

      {/* Hover actions — positioned absolutely so they don't shift layout */}
      <div
        className={cn(
          "absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-0.5 rounded-md border border-border bg-background p-1 shadow-sm",
          "opacity-0 transition-opacity group-hover:opacity-100",
          isSelected && "hidden"
        )}
        onClick={(e) => e.stopPropagation()}
      >
        <ActionButton
          onClick={(e) => handleAction(e, "archive")}
          title={t("inbox.actions.archive")}
          icon={<Archive className="h-3.5 w-3.5" />}
        />
        <ActionButton
          onClick={(e) => handleAction(e, "reply")}
          title={t("inbox.actions.reply")}
          icon={<Reply className="h-3.5 w-3.5" />}
        />
        <ActionButton
          onClick={(e) => handleAction(e, "snooze")}
          title={t("inbox.actions.snooze")}
          icon={<Clock className="h-3.5 w-3.5" />}
        />
        <ActionButton
          onClick={(e) => handleAction(e, "task")}
          title={t("inbox.actions.convert_to_task")}
          icon={<ListTodo className="h-3.5 w-3.5" />}
        />
      </div>
    </div>
  );
}

interface ActionButtonProps {
  onClick: (e: React.MouseEvent) => void;
  title: string;
  icon: React.ReactNode;
}

function ActionButton({ onClick, title, icon }: ActionButtonProps) {
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
