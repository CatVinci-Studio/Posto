import * as React from "react";
import { Routes, Route, Navigate, NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Settings2,
  Mail,
  Sparkles,
  Languages,
  Brain,
  Zap,
  Info,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useSettingsStore } from "@/stores/settingsStore";
import { GeneralTab } from "./GeneralTab";
import { AccountsTab } from "./AccountsTab";
import { LlmTab } from "./LlmTab";
import { LanguageTab } from "./LanguageTab";
import { MemoryTab } from "./MemoryTab";
import { AutomationTab } from "./AutomationTab";

// ---------------------------------------------------------------------------
// Nav items
// ---------------------------------------------------------------------------
interface NavItem {
  to: string;
  labelKey: string;
  icon: React.ReactNode;
}

const NAV_ITEMS: NavItem[] = [
  {
    to: "/settings/general",
    labelKey: "settings.tabs.general",
    icon: <Settings2 className="h-4 w-4 shrink-0" />,
  },
  {
    to: "/settings/accounts",
    labelKey: "settings.tabs.accounts",
    icon: <Mail className="h-4 w-4 shrink-0" />,
  },
  {
    to: "/settings/llm",
    labelKey: "settings.tabs.llm",
    icon: <Sparkles className="h-4 w-4 shrink-0" />,
  },
  {
    to: "/settings/language",
    labelKey: "settings.tabs.language",
    icon: <Languages className="h-4 w-4 shrink-0" />,
  },
  {
    to: "/settings/memory",
    labelKey: "settings.tabs.memory",
    icon: <Brain className="h-4 w-4 shrink-0" />,
  },
  {
    to: "/settings/automation",
    labelKey: "settings.tabs.automation",
    icon: <Zap className="h-4 w-4 shrink-0" />,
  },
];

// ---------------------------------------------------------------------------
// Settings component
// ---------------------------------------------------------------------------
export function Settings() {
  const { t } = useTranslation();
  const { hydrateFromBackend, hydrated } = useSettingsStore();

  React.useEffect(() => {
    if (!hydrated) {
      hydrateFromBackend();
    }
  }, [hydrated, hydrateFromBackend]);

  return (
    <div className="flex h-full overflow-hidden">
      {/* Left nav rail */}
      <nav className="flex w-60 shrink-0 flex-col border-r border-border bg-background py-4 overflow-y-auto">
        <div className="px-4 pb-3">
          <h1 className="text-base font-semibold text-foreground">
            {t("sidebar.settings")}
          </h1>
        </div>

        <div className="flex-1 px-2 space-y-0.5">
          {NAV_ITEMS.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-2.5 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                  isActive
                    ? "bg-accent text-accent-foreground border-l-2 border-primary -ml-0.5 pl-[calc(0.75rem-1px)]"
                    : "text-muted-foreground hover:bg-accent/50 hover:text-foreground"
                )
              }
            >
              {item.icon}
              {t(item.labelKey)}
            </NavLink>
          ))}
        </div>

        {/* About footer item */}
        <div className="px-2 pt-2 border-t border-border mt-2">
          <NavLink
            to="/settings/general"
            className="flex items-center gap-2.5 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent/50 hover:text-foreground transition-colors"
          >
            <Info className="h-4 w-4 shrink-0" />
            {t("settings.tabs.about")}
          </NavLink>
        </div>
      </nav>

      {/* Right pane */}
      <main className="flex-1 overflow-y-auto">
        <div className="mx-auto max-w-2xl px-8 py-8">
          <Routes>
            <Route index element={<Navigate to="general" replace />} />
            <Route path="general" element={<GeneralTab />} />
            <Route path="accounts" element={<AccountsTab />} />
            <Route path="llm" element={<LlmTab />} />
            <Route path="language" element={<LanguageTab />} />
            <Route path="memory" element={<MemoryTab />} />
            <Route path="automation" element={<AutomationTab />} />
            <Route path="*" element={<Navigate to="general" replace />} />
          </Routes>
        </div>
      </main>
    </div>
  );
}
