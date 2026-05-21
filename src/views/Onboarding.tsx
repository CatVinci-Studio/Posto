import { useTranslation } from "react-i18next";

export function Onboarding() {
  const { t } = useTranslation();
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="max-w-md text-center">
        <h1 className="text-2xl font-semibold">{t("onboarding.welcome")}</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          {t("onboarding.add_first_account")}
        </p>
      </div>
    </div>
  );
}
