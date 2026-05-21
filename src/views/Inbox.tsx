import { useTranslation } from "react-i18next";

export function Inbox() {
  const { t } = useTranslation();
  return (
    <div className="flex h-full items-center justify-center text-muted-foreground">
      <div className="text-center">
        <div className="text-4xl">📬</div>
        <p className="mt-3 text-sm">{t("inbox.empty")}</p>
      </div>
    </div>
  );
}
