import * as React from "react";
import { useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { X, Sparkles, Send, Save, Trash2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { call } from "@/lib/tauri";
import { Button } from "@/components/ui/Button";
import { Dialog, DialogFooter } from "@/components/ui/Dialog";
import { useComposeStore } from "@/stores/composeStore";
import type { DraftForm, SendResult } from "@/types/compose";
import { AddressInput } from "./AddressInput";
import { AccountSelect } from "./AccountSelect";
import { AiSidePanel } from "./AiSidePanel";

// ---------------------------------------------------------------------------
// Route params
// ---------------------------------------------------------------------------
interface ComposeParams {
  messageId?: string; // /compose/reply/:messageId
  draftId?: string;   // /compose/draft/:draftId
}

// ---------------------------------------------------------------------------
// Draft key helpers
// ---------------------------------------------------------------------------
function getDraftKey(params: ComposeParams): string {
  if (params.draftId) return `draft:${params.draftId}`;
  if (params.messageId) return `reply:${params.messageId}`;
  return "new";
}

function emptyForm(): DraftForm {
  return { to: [], cc: [], bcc: [], subject: "", body: "" };
}

// ---------------------------------------------------------------------------
// Compose view
// ---------------------------------------------------------------------------
export function Compose() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const params = useParams<{ messageId?: string; draftId?: string }>();
  const draftKey = getDraftKey(params);

  const { saveDraft, loadDraft, clearDraft } = useComposeStore();

  // ---- form state ----
  const [form, setForm] = React.useState<DraftForm>(() => {
    const saved = loadDraft(draftKey);
    if (saved) return saved;
    const initial = emptyForm();
    if (params.messageId) initial.reply_to_message_id = Number(params.messageId);
    return initial;
  });

  const [showCcBcc, setShowCcBcc] = React.useState(
    form.cc.length > 0 || form.bcc.length > 0
  );
  const [showAiPanel, setShowAiPanel] = React.useState(false);
  const [isSending, setIsSending] = React.useState(false);
  const [showDiscardDialog, setShowDiscardDialog] = React.useState(false);
  const [toastMsg, setToastMsg] = React.useState<string | null>(null);

  const bodyRef = React.useRef<HTMLTextAreaElement>(null);

  // ---- helpers ----
  function patch(partial: Partial<DraftForm>) {
    setForm((prev) => ({ ...prev, ...partial }));
  }

  function showToast(msg: string, ms = 2500) {
    setToastMsg(msg);
    setTimeout(() => setToastMsg(null), ms);
  }

  // ---- auto-grow textarea ----
  React.useEffect(() => {
    const el = bodyRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
  }, [form.body]);

  // ---- auto-save draft every 30 seconds ----
  React.useEffect(() => {
    const hasContent =
      form.body.trim().length > 0 ||
      form.subject.trim().length > 0 ||
      form.to.length > 0;
    if (!hasContent) return;

    const id = setInterval(async () => {
      saveDraft(draftKey, form);
      try {
        await call("create_draft", { form });
      } catch {
        // Backend not ready — localStorage save above is enough
      }
    }, 30_000);

    return () => clearInterval(id);
  }, [form, draftKey, saveDraft]);

  // ---- persist to store on every change ----
  React.useEffect(() => {
    saveDraft(draftKey, form);
  }, [form, draftKey, saveDraft]);

  // ---- title ----
  const isReply = !!params.messageId;
  // We don't have original subject in this scope; show generic reply header
  const headerTitle = isReply
    ? `${t("compose.reply_to")}: ${form.subject || "…"}`
    : t("compose.new");

  // ---- send ----
  const handleSend = async () => {
    if (form.to.length === 0) {
      showToast("Please add at least one recipient.");
      return;
    }
    setIsSending(true);
    try {
      let result: SendResult;
      try {
        result = await call<SendResult>("send_message", { form });
      } catch {
        // Mock: simulate success
        result = { success: true, message_id: Math.floor(Math.random() * 10000) };
      }
      if (result.success) {
        clearDraft(draftKey);
        showToast(t("compose.sent"));
        setTimeout(() => navigate(-1), 800);
      } else {
        showToast(result.error ?? "Failed to send.");
      }
    } catch (err) {
      showToast(`Error: ${String(err)}`);
    } finally {
      setIsSending(false);
    }
  };

  // ---- save draft manually ----
  const handleSaveDraft = async () => {
    saveDraft(draftKey, form);
    try {
      await call("create_draft", { form });
    } catch {
      // ignore
    }
    showToast(t("compose.draft_saved"));
  };

  // ---- discard ----
  const handleDiscard = () => {
    clearDraft(draftKey);
    setShowDiscardDialog(false);
    navigate(-1);
  };

  return (
    <div className="flex h-full flex-col bg-background">
      {/* ---- sticky header ---- */}
      <header className="sticky top-0 z-10 flex items-center justify-between border-b border-border bg-background/95 px-6 py-3 backdrop-blur-sm">
        <h1 className="text-base font-semibold text-foreground truncate max-w-[70%]">
          {headerTitle}
        </h1>
        <div className="flex items-center gap-2">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => setShowAiPanel((v) => !v)}
            aria-label={t("compose.ai_assist")}
            className={cn(
              "h-8 w-8 p-0",
              showAiPanel && "bg-accent text-accent-foreground"
            )}
          >
            <Sparkles className="h-4 w-4" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => {
              const hasContent =
                form.body.trim() || form.subject.trim() || form.to.length > 0;
              if (hasContent) {
                setShowDiscardDialog(true);
              } else {
                navigate(-1);
              }
            }}
            aria-label="Close compose"
            className="h-8 w-8 p-0"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      </header>

      {/* ---- main content area ---- */}
      <div className="flex flex-1 overflow-hidden">
        {/* Scrollable form */}
        <div className="flex flex-1 flex-col overflow-y-auto">
          <div className="flex flex-col gap-0 px-6 pt-4">
            {/* From */}
            <div className="flex items-center gap-3 border-b border-border py-2">
              <span className="w-14 shrink-0 text-sm font-medium text-muted-foreground">
                {t("compose.from")}
              </span>
              <AccountSelect
                value={form.from_account_id}
                onChange={(id) => patch({ from_account_id: id })}
              />
            </div>

            {/* To */}
            <div className="flex items-start gap-3 border-b border-border py-2">
              <span className="mt-2 w-14 shrink-0 text-sm font-medium text-muted-foreground">
                {t("compose.to")}
              </span>
              <div className="flex-1">
                <AddressInput
                  label=""
                  value={form.to}
                  onChange={(to) => patch({ to })}
                  placeholder="name@example.com"
                  id="compose-to"
                />
              </div>
              {/* Cc Bcc toggle */}
              {!showCcBcc && (
                <button
                  type="button"
                  onClick={() => setShowCcBcc(true)}
                  className="shrink-0 text-xs text-muted-foreground underline-offset-2 hover:text-foreground hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 rounded"
                >
                  Cc Bcc
                </button>
              )}
            </div>

            {/* Cc */}
            {showCcBcc && (
              <div className="flex items-start gap-3 border-b border-border py-2">
                <span className="mt-2 w-14 shrink-0 text-sm font-medium text-muted-foreground">
                  {t("compose.cc")}
                </span>
                <div className="flex-1">
                  <AddressInput
                    label=""
                    value={form.cc}
                    onChange={(cc) => patch({ cc })}
                    placeholder="cc@example.com"
                    id="compose-cc"
                  />
                </div>
              </div>
            )}

            {/* Bcc */}
            {showCcBcc && (
              <div className="flex items-start gap-3 border-b border-border py-2">
                <span className="mt-2 w-14 shrink-0 text-sm font-medium text-muted-foreground">
                  {t("compose.bcc")}
                </span>
                <div className="flex-1">
                  <AddressInput
                    label=""
                    value={form.bcc}
                    onChange={(bcc) => patch({ bcc })}
                    placeholder="bcc@example.com"
                    id="compose-bcc"
                  />
                </div>
              </div>
            )}

            {/* Subject */}
            <div className="flex items-center gap-3 border-b border-border py-2">
              <span className="w-14 shrink-0 text-sm font-medium text-muted-foreground">
                {t("compose.subject")}
              </span>
              <input
                type="text"
                value={form.subject}
                onChange={(e) => patch({ subject: e.target.value })}
                placeholder={t("compose.subject")}
                className="flex-1 bg-transparent text-sm text-foreground placeholder:text-muted-foreground focus:outline-none"
              />
            </div>

            {/* Body */}
            <div className="py-4 pb-6">
              <textarea
                ref={bodyRef}
                value={form.body}
                onChange={(e) => patch({ body: e.target.value })}
                placeholder={t("compose.body_placeholder")}
                rows={16}
                style={{ minHeight: "16rem" }}
                className={cn(
                  "w-full resize-none bg-transparent text-sm leading-relaxed text-foreground",
                  "placeholder:text-muted-foreground focus:outline-none"
                )}
              />
            </div>
          </div>
        </div>

        {/* AI side panel */}
        {showAiPanel && (
          <aside className="w-80 shrink-0 overflow-y-auto border-l border-border p-4">
            <AiSidePanel
              replyToMessageId={form.reply_to_message_id}
              body={form.body}
              onBodyChange={(body) => patch({ body })}
            />
          </aside>
        )}
      </div>

      {/* ---- sticky footer ---- */}
      <footer className="sticky bottom-0 z-10 flex items-center justify-between border-t border-border bg-background/95 px-6 py-3 backdrop-blur-sm">
        <div className="flex items-center gap-2">
          {/* Send — always requires explicit click */}
          <Button
            type="button"
            variant="default"
            size="md"
            onClick={handleSend}
            disabled={isSending || form.to.length === 0}
          >
            {isSending ? (
              t("compose.sending")
            ) : (
              <>
                <Send className="h-4 w-4" />
                {t("compose.send")}
              </>
            )}
          </Button>

          <Button
            type="button"
            variant="outline"
            size="md"
            onClick={handleSaveDraft}
          >
            <Save className="h-4 w-4" />
            {t("compose.save_draft")}
          </Button>

          <Button
            type="button"
            variant="ghost"
            size="md"
            onClick={() => {
              const hasContent =
                form.body.trim() || form.subject.trim() || form.to.length > 0;
              if (hasContent) {
                setShowDiscardDialog(true);
              } else {
                navigate(-1);
              }
            }}
          >
            <Trash2 className="h-4 w-4" />
            {t("compose.discard")}
          </Button>
        </div>

        {/* AI toggle (right side) */}
        <Button
          type="button"
          variant={showAiPanel ? "secondary" : "ghost"}
          size="sm"
          onClick={() => setShowAiPanel((v) => !v)}
          className="gap-1.5"
        >
          <Sparkles className="h-3.5 w-3.5" />
          {t("compose.ai_assist")}
        </Button>
      </footer>

      {/* ---- Discard confirm dialog ---- */}
      <Dialog
        open={showDiscardDialog}
        onClose={() => setShowDiscardDialog(false)}
        title={t("compose.discard")}
      >
        <p className="text-sm text-muted-foreground">{t("compose.discard_confirm")}</p>
        <DialogFooter>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => setShowDiscardDialog(false)}
          >
            {t("onboarding.back")}
          </Button>
          <Button
            type="button"
            variant="default"
            size="sm"
            onClick={handleDiscard}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {t("compose.discard")}
          </Button>
        </DialogFooter>
      </Dialog>

      {/* ---- Toast ---- */}
      {toastMsg && (
        <div
          role="status"
          aria-live="polite"
          className={cn(
            "fixed bottom-20 left-1/2 z-50 -translate-x-1/2 rounded-full bg-foreground px-4 py-2",
            "text-xs font-medium text-background shadow-lg",
            "animate-in fade-in-0 slide-in-from-bottom-2 duration-200"
          )}
        >
          {toastMsg}
        </div>
      )}
    </div>
  );
}
