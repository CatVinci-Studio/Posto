import * as React from "react";
import { useTranslation } from "react-i18next";
import { ExternalLink } from "lucide-react";
import { Card, CardHeader, CardContent } from "@/components/ui/Card";
import { Select } from "@/components/ui/Select";
import { SegmentedControl } from "@/components/ui/SegmentedControl";
import { useSettingsStore } from "@/stores/settingsStore";
import { call } from "@/lib/tauri";
import type { UiLang, DisplayLang } from "@/types/settings";

export function GeneralTab() {
  const { t } = useTranslation();
  const { ui_lang, display_lang, theme, setUiLang, setDisplayLang, setTheme } =
    useSettingsStore();

  const [appVersion, setAppVersion] = React.useState<string>("0.1.0");

  React.useEffect(() => {
    call<string>("app_version")
      .then(setAppVersion)
      .catch(() => {/* keep default */});
  }, []);

  const themeOptions = [
    { value: "system" as const, label: t("settings.general.theme_system") },
    { value: "light" as const, label: t("settings.general.theme_light") },
    { value: "dark" as const, label: t("settings.general.theme_dark") },
  ];

  return (
    <div className="space-y-6">
      {/* Language & Theme */}
      <Card>
        <CardHeader>
          <h2 className="text-base font-semibold">{t("settings.tabs.general")}</h2>
          <p className="text-xs text-muted-foreground">
            {t("settings.general.ui_lang")} &amp; {t("settings.general.display_lang")}
          </p>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="grid grid-cols-1 gap-6 sm:grid-cols-2">
            {/* UI Language */}
            <div className="space-y-1.5">
              <label className="text-sm font-medium">{t("settings.general.ui_lang")}</label>
              <Select
                value={ui_lang}
                onChange={(e) => setUiLang(e.target.value as UiLang)}
              >
                <option value="en">English</option>
                <option value="zh">中文</option>
              </Select>
              <p className="text-xs text-muted-foreground">
                {t("settings.language.ui_lang_desc")}
              </p>
            </div>

            {/* Display Language */}
            <div className="space-y-1.5">
              <label className="text-sm font-medium">{t("settings.general.display_lang")}</label>
              <Select
                value={display_lang}
                onChange={(e) => setDisplayLang(e.target.value as DisplayLang)}
              >
                <option value="original">{t("settings.language.display_lang_original")}</option>
                <option value="en">English</option>
                <option value="zh">中文</option>
                <option value="ja">日本語</option>
              </Select>
              <p className="text-xs text-muted-foreground">
                {t("settings.language.display_lang_desc")}
              </p>
            </div>
          </div>

          {/* Theme */}
          <div className="space-y-1.5">
            <label className="text-sm font-medium">{t("settings.general.theme")}</label>
            <SegmentedControl
              options={themeOptions}
              value={theme}
              onChange={setTheme}
            />
          </div>
        </CardContent>
      </Card>

      {/* About */}
      <Card>
        <CardHeader>
          <h2 className="text-base font-semibold">{t("settings.tabs.about")}</h2>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Version</span>
            <span className="font-mono text-xs bg-muted px-2 py-0.5 rounded">{appVersion}</span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">License</span>
            <span>MIT</span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Source</span>
            <a
              href="https://github.com/retposto/retposto"
              target="_blank"
              rel="noreferrer"
              className="flex items-center gap-1 text-primary hover:underline"
            >
              GitHub
              <ExternalLink className="h-3 w-3" />
            </a>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
