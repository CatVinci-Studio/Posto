import { NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Inbox as InboxIcon,
  Calendar,
  Newspaper,
  CheckCircle2,
  Sparkles,
  Settings as SettingsIcon,
  Plus,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface NavItem {
  to: string;
  labelKey: string;
  Icon: React.ComponentType<{ className?: string }>;
}

const PRIMARY: NavItem[] = [
  { to: "/inbox", labelKey: "sidebar.all_inboxes", Icon: InboxIcon },
  { to: "/inbox?filter=today", labelKey: "sidebar.today", Icon: Calendar },
  { to: "/inbox?filter=newsletters", labelKey: "sidebar.newsletters", Icon: Newspaper },
  { to: "/inbox?filter=done", labelKey: "sidebar.done", Icon: CheckCircle2 },
];

export function Sidebar() {
  const { t } = useTranslation();
  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-border bg-muted/30 p-3">
      <div className="px-2 py-3 text-lg font-semibold tracking-tight">
        {t("app.name")}
      </div>

      <nav className="mt-2 flex flex-col gap-1">
        {PRIMARY.map(({ to, labelKey, Icon }) => (
          <NavLink
            key={to}
            to={to}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
              )
            }
          >
            <Icon className="h-4 w-4" />
            <span>{t(labelKey)}</span>
          </NavLink>
        ))}
      </nav>

      <div className="mt-6 px-3 text-xs uppercase tracking-wider text-muted-foreground">
        {t("sidebar.accounts")}
      </div>
      <div className="mt-2 flex-1 overflow-auto px-1">
        <button
          type="button"
          className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent/60 hover:text-foreground"
          onClick={() => (window.location.href = "/onboarding")}
        >
          <Plus className="h-4 w-4" />
          {t("sidebar.add_account")}
        </button>
      </div>

      <div className="mt-2 flex flex-col gap-1 border-t border-border pt-2">
        <NavLink
          to="/inbox?activity=1"
          className="flex items-center gap-2 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent/60 hover:text-foreground"
        >
          <Sparkles className="h-4 w-4" />
          {t("sidebar.agent_activity")}
        </NavLink>
        <NavLink
          to="/settings"
          className={({ isActive }) =>
            cn(
              "flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
              isActive
                ? "bg-accent text-accent-foreground"
                : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
            )
          }
        >
          <SettingsIcon className="h-4 w-4" />
          {t("sidebar.settings")}
        </NavLink>
      </div>
    </aside>
  );
}
