import { useTranslation } from "react-i18next";

export function Settings() {
  const { t } = useTranslation();
  return (
    <div className="p-6">
      <h1 className="text-2xl font-semibold">{t("sidebar.settings")}</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        {t("settings.general")} · {t("settings.accounts")} · {t("settings.llm")} ·{" "}
        {t("settings.memory")} · {t("settings.automation")}
      </p>
    </div>
  );
}
