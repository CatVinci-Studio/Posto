import * as React from "react";
import { useTranslation } from "react-i18next";
import { Loader2 } from "lucide-react";
import { Card, CardContent } from "@/components/ui/Card";

interface Props {
  onSuccess: () => void;
}

/**
 * Onboarding completion screen. The actual IMAP / OAuth connection test
 * happens inside `AuthStep` (test_imap_login or begin_oauth_login →
 * handle_oauth_callback). By the time we reach this step the credentials
 * have already been verified and persisted in the keychain; this is the
 * brief "all set" animation before showing SuccessStep.
 */
export function TestStep({ onSuccess }: Props) {
  const { t } = useTranslation();

  React.useEffect(() => {
    const timer = setTimeout(() => {
      onSuccess();
    }, 800);
    return () => clearTimeout(timer);
  }, [onSuccess]);

  return (
    <Card className="w-full">
      <CardContent className="flex flex-col items-center justify-center gap-4 py-16">
        <Loader2 className="h-10 w-10 animate-spin text-primary" />
        <p className="text-sm text-muted-foreground">{t("onboarding.testing_connection")}</p>
      </CardContent>
    </Card>
  );
}
