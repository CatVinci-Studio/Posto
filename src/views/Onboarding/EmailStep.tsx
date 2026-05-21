import * as React from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/Card";
import { detectProvider, type ProviderKind } from "./detect";

interface Props {
  email: string;
  onChange: (email: string) => void;
  onNext: (provider: ProviderKind | null) => void;
}

export function EmailStep({ email, onChange, onNext }: Props) {
  const { t } = useTranslation();
  const [error, setError] = React.useState<string | undefined>();

  function handleNext() {
    const trimmed = email.trim();
    if (!trimmed || !trimmed.includes("@")) {
      setError("Please enter a valid email address.");
      return;
    }
    setError(undefined);
    const provider = detectProvider(trimmed);
    onNext(provider);
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") handleNext();
  }

  return (
    <Card className="w-full">
      <CardHeader>
        <h1 className="text-2xl font-semibold">{t("onboarding.welcome")}</h1>
        <p className="text-sm text-muted-foreground">
          {t("onboarding.add_first_account")}
        </p>
      </CardHeader>
      <CardContent>
        <Input
          label={t("onboarding.email_address")}
          type="email"
          autoFocus
          autoComplete="email"
          placeholder="you@example.com"
          value={email}
          onChange={(e) => {
            onChange(e.target.value);
            if (error) setError(undefined);
          }}
          onKeyDown={handleKeyDown}
          error={error}
        />
      </CardContent>
      <CardFooter className="flex-col items-stretch gap-3">
        <Button size="lg" className="w-full" onClick={handleNext}>
          {t("onboarding.next")}
        </Button>
      </CardFooter>
    </Card>
  );
}
