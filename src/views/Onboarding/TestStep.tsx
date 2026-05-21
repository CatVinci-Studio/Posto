import * as React from "react";
import { useTranslation } from "react-i18next";
import { Loader2 } from "lucide-react";
import { Card, CardContent } from "@/components/ui/Card";

interface Props {
  onSuccess: () => void;
}

export function TestStep({ onSuccess }: Props) {
  const { t } = useTranslation();

  React.useEffect(() => {
    // TODO: replace with real IPC call to test_imap_login (password-based providers)
    // or call('finalize_oauth_account', { ... }) for OAuth providers
    const timer = setTimeout(() => {
      onSuccess();
    }, 1500);
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
