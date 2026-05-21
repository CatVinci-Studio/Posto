import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X, Sparkles, Reply, Archive, ListTodo, ChevronDown, ChevronUp } from "lucide-react";
import { call } from "@/lib/tauri";
import { useInboxStore } from "@/stores/inboxStore";
import { Card } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { Avatar } from "./Avatar";
import { CategoryChip } from "./CategoryChip";
import type { InboxItem } from "@/types/email";

// --- Mock suggested tasks ---
const MOCK_TASKS = [
  "Review and approve revenue slide",
  "Add comments to Q3 projections",
  "Share final deck with the board",
];

interface MessageDetailProps {
  item: InboxItem;
}

export function MessageDetail({ item }: MessageDetailProps) {
  const { t, i18n } = useTranslation();
  const { setSelected } = useInboxStore();
  const { message, enrichment } = item;

  const [showCc, setShowCc] = useState(false);
  const [translatedBody, setTranslatedBody] = useState<string | null>(null);
  const [isTranslating, setIsTranslating] = useState(false);
  const [showTranslated, setShowTranslated] = useState(false);

  const toAddrs: string[] = message.to_addrs ? JSON.parse(message.to_addrs) : [];
  const ccAddrs: string[] = message.cc_addrs ? JSON.parse(message.cc_addrs) : [];

  const dateStr = message.date
    ? new Date(message.date * 1000).toLocaleString(
        i18n.language === "zh" ? "zh-CN" : "en-US",
        {
          weekday: "short",
          year: "numeric",
          month: "short",
          day: "numeric",
          hour: "2-digit",
          minute: "2-digit",
        }
      )
    : "";

  async function handleTranslate() {
    if (translatedBody) {
      setShowTranslated((v) => !v);
      return;
    }
    setIsTranslating(true);
    try {
      const result = await call<string>("translate_message", {
        message_id: message.id,
        target_lang: i18n.language,
      });
      setTranslatedBody(result);
      setShowTranslated(true);
    } catch {
      setTranslatedBody("(translation mocked — backend not connected)");
      setShowTranslated(true);
    } finally {
      setIsTranslating(false);
    }
  }

  function handleReply() {
    // TODO: open compose pane with reply pre-filled
    console.log("reply to", message.id);
  }

  function handleArchive() {
    // TODO: call('archive_message', { message_id: message.id })
    console.log("archive", message.id);
    setSelected(null);
  }

  function handleConvertToTask() {
    // TODO: call('create_task_from_message', { message_id: message.id })
    console.log("convert to task", message.id);
  }

  const bodyContent = showTranslated && translatedBody
    ? translatedBody
    : (message.body_text ?? message.snippet ?? "(no body)");

  const taskCount = enrichment?.task_count ?? 0;
  const mockTasks = MOCK_TASKS.slice(0, Math.min(taskCount, MOCK_TASKS.length));

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {/* Header bar */}
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-2 min-w-0">
          {enrichment?.category && (
            <CategoryChip category={enrichment.category} />
          )}
          <h1 className="truncate text-base font-semibold text-foreground">
            {message.subject ?? "(no subject)"}
          </h1>
        </div>
        <button
          type="button"
          aria-label="Close"
          onClick={() => setSelected(null)}
          className="ml-2 flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      <div className="flex min-h-0 flex-1">
        {/* Main content area */}
        <div className="flex-1 overflow-y-auto px-5 py-4">
          {/* Sender info */}
          <div className="flex items-start gap-3">
            <Avatar name={message.from_name} email={message.from_addr} size="lg" />
            <div className="min-w-0 flex-1">
              <div className="font-medium text-foreground">
                {message.from_name ?? message.from_addr}
              </div>
              {message.from_name && (
                <div className="text-xs text-muted-foreground">{message.from_addr}</div>
              )}
              {toAddrs.length > 0 && (
                <div className="mt-0.5 text-xs text-muted-foreground">
                  <span className="font-medium">{t("inbox.detail.to")}: </span>
                  {toAddrs.join(", ")}
                </div>
              )}
              {ccAddrs.length > 0 && (
                <div className="mt-0.5 text-xs text-muted-foreground">
                  <button
                    type="button"
                    className="inline-flex items-center gap-0.5 hover:text-foreground transition-colors"
                    onClick={() => setShowCc((v) => !v)}
                  >
                    <span className="font-medium">{t("inbox.detail.cc")}</span>
                    {showCc ? (
                      <ChevronUp className="h-3 w-3" />
                    ) : (
                      <ChevronDown className="h-3 w-3" />
                    )}
                  </button>
                  {showCc && <span className="ml-1">{ccAddrs.join(", ")}</span>}
                </div>
              )}
              <div className="mt-0.5 text-xs text-muted-foreground">{dateStr}</div>
            </div>
          </div>

          {/* AI Summary card */}
          {enrichment?.summary && (
            <Card className="mt-4 flex items-start gap-3 px-4 py-3 bg-accent/30 border-accent/60">
              <Sparkles className="mt-0.5 h-4 w-4 shrink-0 text-primary" />
              <div>
                <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1">
                  {t("inbox.detail.summary")}
                </p>
                <p className="text-sm text-foreground leading-relaxed">{enrichment.summary}</p>
                {enrichment.facts && enrichment.facts.length > 0 && (
                  <ul className="mt-2 space-y-0.5">
                    {enrichment.facts.map((fact, i) => (
                      <li key={i} className="flex items-start gap-1.5 text-xs text-muted-foreground">
                        <span className="mt-1 h-1.5 w-1.5 shrink-0 rounded-full bg-primary/60" />
                        {fact}
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </Card>
          )}

          {/* Email body */}
          <div className="mt-5">
            <pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed text-foreground">
              {bodyContent}
            </pre>
            {/* TODO: V2 — render body_html with proper DOMPurify sanitization */}
          </div>
        </div>

        {/* Suggested tasks side panel */}
        {mockTasks.length > 0 && (
          <div className="w-56 shrink-0 border-l border-border bg-muted/20 px-3 py-4">
            <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-3">
              {t("inbox.detail.suggested_tasks")}
            </p>
            <ul className="space-y-2">
              {mockTasks.map((task, i) => (
                <li
                  key={i}
                  className="flex items-start gap-2 rounded-md bg-background p-2 text-xs text-foreground border border-border"
                >
                  <ListTodo className="mt-0.5 h-3 w-3 shrink-0 text-primary" />
                  {task}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {/* Footer toolbar */}
      <div className="flex items-center gap-2 border-t border-border px-4 py-3">
        <Button size="sm" onClick={handleReply}>
          <Reply className="h-4 w-4" />
          {t("inbox.actions.reply")}
        </Button>
        <Button size="sm" variant="outline" onClick={handleTranslate} disabled={isTranslating}>
          {isTranslating
            ? "…"
            : showTranslated
            ? t("inbox.detail.original")
            : t("inbox.detail.toggle_translation")}
        </Button>
        <div className="flex-1" />
        <Button
          size="sm"
          variant="ghost"
          onClick={handleConvertToTask}
          title={t("inbox.actions.convert_to_task")}
        >
          <ListTodo className="h-4 w-4" />
          <span className="hidden sm:inline">{t("inbox.actions.convert_to_task")}</span>
        </Button>
        <Button
          size="sm"
          variant="ghost"
          onClick={handleArchive}
          title={t("inbox.actions.archive")}
        >
          <Archive className="h-4 w-4" />
          <span className="hidden sm:inline">{t("inbox.actions.archive")}</span>
        </Button>
      </div>
    </div>
  );
}
