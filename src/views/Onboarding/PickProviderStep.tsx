import * as React from "react";
import { useTranslation } from "react-i18next";
import { Mail, Server } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardHeader } from "@/components/ui/Card";
import { cn } from "@/lib/utils";
import type { ProviderKind } from "./detect";

interface ProviderTile {
  id: ProviderKind;
  nameKey: string;
  iconColor: string;
  Icon: React.ComponentType<{ className?: string }>;
}

// TODO: replace with brand SVGs
const PROVIDERS: ProviderTile[] = [
  { id: "gmail",        nameKey: "onboarding.providers.gmail",       iconColor: "text-red-500",   Icon: Mail },
  { id: "outlook",      nameKey: "onboarding.providers.outlook",     iconColor: "text-blue-500",  Icon: Mail },
  { id: "icloud",       nameKey: "onboarding.providers.icloud",      iconColor: "text-slate-500", Icon: Mail },
  { id: "qq",           nameKey: "onboarding.providers.qq",          iconColor: "text-blue-400",  Icon: Mail },
  { id: "mail163",      nameKey: "onboarding.providers.mail163",     iconColor: "text-red-600",   Icon: Mail },
  { id: "generic_imap", nameKey: "onboarding.providers.generic_imap", iconColor: "text-foreground", Icon: Server },
];

interface Props {
  onSelect: (provider: ProviderKind) => void;
  onBack: () => void;
}

export function PickProviderStep({ onSelect, onBack }: Props) {
  const { t } = useTranslation();

  return (
    <Card className="w-full">
      <CardHeader>
        <h1 className="text-2xl font-semibold">{t("onboarding.pick_provider")}</h1>
        <p className="text-sm text-muted-foreground">
          {t("onboarding.add_first_account")}
        </p>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-3 gap-3">
          {PROVIDERS.map(({ id, nameKey, iconColor, Icon }) => (
            <button
              key={id}
              onClick={() => onSelect(id)}
              className={cn(
                "flex flex-col items-center gap-2 rounded-lg border border-border p-4",
                "hover:bg-accent hover:text-accent-foreground transition-colors",
                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              )}
            >
              <Icon className={cn("h-7 w-7", iconColor)} />
              <span className="text-xs font-medium leading-tight text-center">
                {t(nameKey)}
              </span>
            </button>
          ))}
        </div>
        <div className="mt-6">
          <Button variant="ghost" size="sm" onClick={onBack} className="w-full">
            {t("onboarding.back")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
