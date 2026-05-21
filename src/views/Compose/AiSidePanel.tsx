import * as React from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import {
  Wand2,
  Languages,
  Pencil,
  Loader2,
  Info,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { call } from "@/lib/tauri";
import { Button } from "@/components/ui/Button";

type Tone = "casual" | "neutral" | "formal";

interface AiSidePanelProps {
  replyToMessageId?: number;
  body: string;
  onBodyChange: (body: string) => void;
  llmConfigured?: boolean;
  className?: string;
}

/**
 * Collapsible AI assistance panel. Mocks all LLM calls with a 1.5s delay.
 */
export function AiSidePanel({
  replyToMessageId,
  body,
  onBodyChange,
  llmConfigured = true,
  className,
}: AiSidePanelProps) {
  const { t } = useTranslation();
  const [tone, setTone] = React.useState<Tone>("neutral");
  const [targetLang, setTargetLang] = React.useState("zh");
  const [loading, setLoading] = React.useState<string | null>(null);

  // Mock delay helper
  async function mockDelay<T>(value: T, ms = 1500): Promise<T> {
    return new Promise((resolve) => setTimeout(() => resolve(value), ms));
  }

  const handleGenerateReply = async () => {
    if (!replyToMessageId) return;
    setLoading("generate");
    try {
      let result: string;
      try {
        result = await call<string>("generate_reply_draft", {
          message_id: replyToMessageId,
        });
      } catch {
        result = await mockDelay(
          `Hi,\n\nThank you for reaching out. I've reviewed your message and wanted to follow up.\n\n[Generated reply placeholder — configure your LLM for a real draft.]\n\nBest regards`
        );
      }
      onBodyChange(result);
    } finally {
      setLoading(null);
    }
  };

  const handleImproveWriting = async () => {
    if (!body.trim()) return;
    setLoading("improve");
    try {
      let result: string;
      try {
        result = await call<string>("improve_writing", { body, tone });
      } catch {
        result = await mockDelay(
          `${body}\n\n[Writing improved with ${tone} tone — configure your LLM for a real result.]`
        );
      }
      onBodyChange(result);
    } finally {
      setLoading(null);
    }
  };

  const handleTranslate = async () => {
    if (!body.trim()) return;
    setLoading("translate");
    try {
      let result: string;
      try {
        result = await call<string>("translate_body", { body, target_lang: targetLang });
      } catch {
        result = await mockDelay(
          `[Translated to ${targetLang} — configure your LLM for a real result.]\n\n${body}`
        );
      }
      onBodyChange(result);
    } finally {
      setLoading(null);
    }
  };

  const tones: { value: Tone; label: string; key: string }[] = [
    { value: "casual", label: t("compose.ai.tone_casual"), key: "casual" },
    { value: "neutral", label: t("compose.ai.tone_neutral"), key: "neutral" },
    { value: "formal", label: t("compose.ai.tone_formal"), key: "formal" },
  ];

  if (!llmConfigured) {
    return (
      <aside
        className={cn(
          "flex flex-col gap-4 rounded-xl border border-border bg-muted/30 p-5 text-sm",
          className
        )}
      >
        <div className="flex items-start gap-2 text-muted-foreground">
          <Info className="mt-0.5 h-4 w-4 shrink-0" />
          <p>{t("compose.ai.ai_not_configured")}</p>
        </div>
        <Link
          to="/settings/llm"
          className="text-xs text-primary underline-offset-2 hover:underline"
        >
          {t("settings.llm")} →
        </Link>
      </aside>
    );
  }

  return (
    <aside
      className={cn(
        "flex flex-col gap-4 rounded-xl border border-border bg-muted/30 p-5 text-sm",
        className
      )}
    >
      {/* Detected language */}
      <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
        <Languages className="h-3.5 w-3.5" />
        <span>
          {t("compose.ai.detected_lang")}: <span className="font-medium text-foreground">English</span>
        </span>
      </div>

      <hr className="border-border" />

      {/* Tone picker */}
      <div>
        <p className="mb-1.5 text-xs font-medium text-muted-foreground">{t("compose.ai.tone")}</p>
        <div className="flex gap-1.5">
          {tones.map((t_) => (
            <button
              key={t_.key}
              type="button"
              onClick={() => setTone(t_.value)}
              className={cn(
                "rounded-full border px-2.5 py-0.5 text-xs transition-colors",
                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1",
                tone === t_.value
                  ? "border-primary bg-primary/10 text-primary"
                  : "border-border bg-background text-muted-foreground hover:bg-accent"
              )}
            >
              {t_.label}
            </button>
          ))}
        </div>
      </div>

      {/* Generate reply */}
      {replyToMessageId && (
        <Button
          type="button"
          variant="secondary"
          size="sm"
          onClick={handleGenerateReply}
          disabled={loading !== null}
          className="justify-start gap-2"
        >
          {loading === "generate" ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <Pencil className="h-3.5 w-3.5" />
          )}
          {t("compose.ai.generate_reply")}
        </Button>
      )}

      {/* Improve writing */}
      <Button
        type="button"
        variant="secondary"
        size="sm"
        onClick={handleImproveWriting}
        disabled={loading !== null || !body.trim()}
        className="justify-start gap-2"
      >
        {loading === "improve" ? (
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
        ) : (
          <Wand2 className="h-3.5 w-3.5" />
        )}
        {t("compose.ai.improve_writing")}
      </Button>

      <hr className="border-border" />

      {/* Translate */}
      <div className="flex flex-col gap-2">
        <p className="text-xs font-medium text-muted-foreground">{t("compose.ai.translate_to")}</p>
        <div className="flex gap-2">
          <select
            value={targetLang}
            onChange={(e) => setTargetLang(e.target.value)}
            className={cn(
              "flex-1 h-8 appearance-none rounded-md border border-border bg-background pl-2 pr-6 text-xs text-foreground",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
            )}
          >
            <option value="zh">中文 (Chinese)</option>
            <option value="en">English</option>
            <option value="es">Español</option>
            <option value="fr">Français</option>
            <option value="de">Deutsch</option>
            <option value="ja">日本語</option>
            <option value="ko">한국어</option>
          </select>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleTranslate}
            disabled={loading !== null || !body.trim()}
            className="h-8 px-3"
          >
            {loading === "translate" ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <Languages className="h-3.5 w-3.5" />
            )}
          </Button>
        </div>
      </div>
    </aside>
  );
}
