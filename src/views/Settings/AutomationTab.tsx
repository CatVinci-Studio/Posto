import * as React from "react";
import { useTranslation } from "react-i18next";
import { Zap, Shield, BookOpen, Lock, Plus } from "lucide-react";
import { Card, CardHeader, CardContent, CardFooter } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { SegmentedControl } from "@/components/ui/SegmentedControl";
import { useSettingsStore } from "@/stores/settingsStore";
import type { TrustLevel, AppSettings } from "@/types/settings";

// ---------------------------------------------------------------------------
// Trust level explanations
// ---------------------------------------------------------------------------
const TRUST_DESCS: Record<TrustLevel, { icon: React.ReactNode; color: string }> = {
  aggressive: {
    icon: <Zap className="h-4 w-4" />,
    color: "border-amber-400 bg-amber-50 dark:bg-amber-950/40",
  },
  medium: {
    icon: <BookOpen className="h-4 w-4" />,
    color: "border-blue-400 bg-blue-50 dark:bg-blue-950/40",
  },
  conservative: {
    icon: <Shield className="h-4 w-4" />,
    color: "border-green-400 bg-green-50 dark:bg-green-950/40",
  },
};

// ---------------------------------------------------------------------------
// Per-action rows
// ---------------------------------------------------------------------------
type PerAction = keyof AppSettings["per_action"];

interface ActionRowProps {
  actionKey: PerAction;
  value: "auto" | "ask";
  onChange: (value: "auto" | "ask") => void;
}

function ActionRow({ actionKey, value, onChange }: ActionRowProps) {
  const { t } = useTranslation();
  return (
    <div className="flex items-center justify-between py-2.5 border-b border-border last:border-0">
      <div>
        <p className="text-sm font-medium">
          {t(`settings.automation.actions.${actionKey}`)}
        </p>
      </div>
      <SegmentedControl
        options={[
          { value: "auto", label: "Auto" },
          { value: "ask", label: "Ask" },
        ]}
        value={value}
        onChange={onChange}
      />
    </div>
  );
}

interface ImmutableActionRowProps {
  label: string;
  badge: string;
}

function ImmutableActionRow({ label, badge }: ImmutableActionRowProps) {
  return (
    <div className="flex items-center justify-between py-2.5 border-b border-border last:border-0 opacity-70">
      <div className="flex items-center gap-2">
        <Lock className="h-3.5 w-3.5 text-muted-foreground" />
        <p className="text-sm font-medium">{label}</p>
      </div>
      <span className="inline-flex items-center rounded-full bg-muted px-2.5 py-0.5 text-xs text-muted-foreground">
        {badge}
      </span>
    </div>
  );
}

// ---------------------------------------------------------------------------
// AutomationTab
// ---------------------------------------------------------------------------
export function AutomationTab() {
  const { t } = useTranslation();
  const { trust_level, per_action, setTrustLevel, setPerAction } = useSettingsStore();
  const [undoKeep, setUndoKeep] = React.useState(50);
  const [comingSoon, setComingSoon] = React.useState(false);

  const trustOptions = [
    {
      value: "aggressive" as TrustLevel,
      label: t("settings.automation.trust_level.aggressive"),
      icon: <Zap className="h-3.5 w-3.5" />,
    },
    {
      value: "medium" as TrustLevel,
      label: t("settings.automation.trust_level.medium"),
      icon: <BookOpen className="h-3.5 w-3.5" />,
    },
    {
      value: "conservative" as TrustLevel,
      label: t("settings.automation.trust_level.conservative"),
      icon: <Shield className="h-3.5 w-3.5" />,
    },
  ];

  const trustInfo = TRUST_DESCS[trust_level];

  return (
    <div className="space-y-6">
      {/* Trust Level */}
      <Card>
        <CardHeader>
          <h2 className="text-base font-semibold">{t("settings.tabs.automation")}</h2>
          <p className="text-xs text-muted-foreground">
            Control how much autonomy the AI agent has
          </p>
        </CardHeader>
        <CardContent className="space-y-4">
          <SegmentedControl
            options={trustOptions}
            value={trust_level}
            onChange={setTrustLevel}
          />
          {/* Trust level explanation card */}
          <div className={`flex items-start gap-3 rounded-lg border p-3 ${trustInfo.color}`}>
            <span className="mt-0.5">{trustInfo.icon}</span>
            <p className="text-sm">
              {t(`settings.automation.trust_level_desc.${trust_level}`)}
            </p>
          </div>
        </CardContent>
      </Card>

      {/* Per-action toggles */}
      <Card>
        <CardHeader>
          <h2 className="text-base font-semibold">Per-Action Settings</h2>
          <p className="text-xs text-muted-foreground">
            Override the global trust level for individual actions
          </p>
        </CardHeader>
        <CardContent className="divide-y-0">
          {(Object.keys(per_action) as PerAction[]).map((key) => (
            <ActionRow
              key={key}
              actionKey={key}
              value={per_action[key]}
              onChange={(v) => setPerAction(key, v)}
            />
          ))}
          {/* Immutable rows */}
          <ImmutableActionRow
            label={t("settings.automation.actions.send")}
            badge={t("settings.automation.always_confirm")}
          />
          <ImmutableActionRow
            label={t("settings.automation.actions.delete_action")}
            badge={t("settings.automation.always_confirm")}
          />
        </CardContent>
      </Card>

      {/* Undo settings */}
      <Card>
        <CardHeader>
          <h2 className="text-base font-semibold">{t("settings.automation.undo_keep")}</h2>
          <p className="text-xs text-muted-foreground">
            Number of recent agent actions kept in the undo history
          </p>
        </CardHeader>
        <CardContent>
          <Input
            type="number"
            value={undoKeep}
            onChange={(e) => setUndoKeep(Number(e.target.value))}
            className="w-24"
            min={1}
            max={500}
          />
        </CardContent>
        <CardFooter className="bg-muted/50 rounded-b-xl justify-end">
          <Button
            variant="default"
            size="sm"
            onClick={() => {/* persist undo_keep */}}
          >
            Save
          </Button>
        </CardFooter>
      </Card>

      {/* Rules */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-base font-semibold">{t("settings.automation.rules")}</h2>
              <p className="text-xs text-muted-foreground">
                Learned or manually defined automation rules
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              className="gap-1.5"
              onClick={() => {
                setComingSoon(true);
                setTimeout(() => setComingSoon(false), 2500);
              }}
            >
              <Plus className="h-4 w-4" />
              {t("settings.automation.add_rule")}
            </Button>
          </div>
          {comingSoon && (
            <p className="text-xs text-muted-foreground mt-1">Coming soon — rule creation will be supported in a future release.</p>
          )}
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground text-center py-6">
            {t("settings.automation.no_rules")}
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
