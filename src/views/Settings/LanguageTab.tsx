import * as React from "react";
import { useTranslation } from "react-i18next";
import { Info } from "lucide-react";
import { Card, CardHeader, CardContent } from "@/components/ui/Card";
import { Select } from "@/components/ui/Select";
import { useSettingsStore } from "@/stores/settingsStore";
import type { UiLang, DisplayLang, ReplyLang } from "@/types/settings";

interface TooltipProps {
  content: string;
}

function InfoTooltip({ content }: TooltipProps) {
  const [visible, setVisible] = React.useState(false);
  return (
    <span className="relative inline-flex">
      <button
        type="button"
        className="text-muted-foreground hover:text-foreground"
        onMouseEnter={() => setVisible(true)}
        onMouseLeave={() => setVisible(false)}
        onFocus={() => setVisible(true)}
        onBlur={() => setVisible(false)}
      >
        <Info className="h-3.5 w-3.5" />
      </button>
      {visible && (
        <span className="absolute left-full top-1/2 z-50 ml-2 -translate-y-1/2 w-56 rounded-md border border-border bg-background p-2.5 text-xs text-foreground shadow-lg">
          {content}
        </span>
      )}
    </span>
  );
}

interface LangSectionProps {
  title: string;
  description: string;
  children: React.ReactNode;
  badge?: React.ReactNode;
}

function LangSection({ title, description, children, badge }: LangSectionProps) {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <h2 className="text-base font-semibold">{title}</h2>
          {badge}
        </div>
        <p className="text-xs text-muted-foreground">{description}</p>
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}

export function LanguageTab() {
  const { t } = useTranslation();
  const { ui_lang, display_lang, reply_lang, setUiLang, setDisplayLang, setReplyLang } =
    useSettingsStore();

  return (
    <div className="space-y-6">
      {/* 1. UI Language */}
      <LangSection
        title={t("settings.general.ui_lang")}
        description={t("settings.language.ui_lang_desc")}
      >
        <Select
          value={ui_lang}
          onChange={(e) => setUiLang(e.target.value as UiLang)}
          className="max-w-xs"
        >
          <option value="en">English</option>
          <option value="zh">中文</option>
        </Select>
      </LangSection>

      {/* 2. Display Language */}
      <LangSection
        title={t("settings.general.display_lang")}
        description={t("settings.language.display_lang_desc")}
      >
        <Select
          value={display_lang}
          onChange={(e) => setDisplayLang(e.target.value as DisplayLang)}
          className="max-w-xs"
        >
          <option value="original">{t("settings.language.display_lang_original")}</option>
          <option value="en">English</option>
          <option value="zh">中文</option>
          <option value="ja">日本語</option>
        </Select>
      </LangSection>

      {/* 3. Reply Language */}
      <LangSection
        title="Reply Language"
        description={t("settings.language.reply_lang_desc")}
      >
        <Select
          value={reply_lang}
          onChange={(e) => setReplyLang(e.target.value as ReplyLang)}
          className="max-w-xs"
        >
          <option value="auto">{t("settings.language.reply_lang.auto")}</option>
          <option value="en">English</option>
          <option value="zh">中文</option>
          <option value="ja">日本語</option>
        </Select>
        <p className="mt-2 text-xs text-muted-foreground">
          {t("settings.language.reply_lang.auto")} is recommended — it automatically matches the sender's language.
        </p>
      </LangSection>

      {/* 4. Prompt Language (read-only) */}
      <LangSection
        title="Prompt Language"
        description={t("settings.language.prompt_lang_desc")}
        badge={
          <InfoTooltip
            content="All internal AI prompts are written in English. This is fixed to maximize reliability and consistency across models. It does not affect the language of your UI, email display, or replies."
          />
        }
      >
        <div className="flex items-center gap-3">
          <div className="flex h-10 items-center rounded-md border border-border bg-muted/50 px-3 text-sm text-muted-foreground select-none">
            English (fixed)
          </div>
          <span className="text-xs text-muted-foreground">
            Cannot be changed
          </span>
        </div>
      </LangSection>
    </div>
  );
}
