import { NavLink, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  Inbox as InboxIcon,
  Calendar,
  Newspaper,
  CheckCircle2,
  Sparkles,
  Settings as SettingsIcon,
  Plus,
  PenSquare,
  X,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/Button";
import { useScreenSize } from "@/hooks/useMediaQuery";

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

interface SidebarProps {
  /** Controlled open state — required on mobile, ignored on desktop. */
  open?: boolean;
  onClose?: () => void;
}

/**
 * Apple Mail-styled sidebar.
 *
 * - Desktop (≥768px): persistent, 240px wide, always visible.
 * - Mobile (<768px): full-height drawer slid in from the left over a scrim.
 */
export function Sidebar({ open = false, onClose }: SidebarProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { isMobile } = useScreenSize();

  const handleNav = (to: string) => {
    navigate(to);
    if (isMobile) onClose?.();
  };

  const content = (
    <aside
      className={cn(
        "flex shrink-0 flex-col bg-muted/40 p-3 no-select",
        // Desktop: persistent rail.
        !isMobile && "h-full w-60 border-r border-border",
        // Mobile: full-height drawer.
        isMobile &&
          "fixed left-0 top-0 z-50 h-full w-72 border-r border-border bg-background shadow-2xl transition-transform duration-200",
        isMobile && (open ? "translate-x-0" : "-translate-x-full")
      )}
      style={isMobile ? { paddingTop: "calc(env(safe-area-inset-top, 0px) + 12px)" } : undefined}
    >
      <div className="flex items-center justify-between px-2 py-2">
        <span className="text-lg font-semibold tracking-tight">
          {t("app.name")}
        </span>
        {isMobile && (
          <button
            type="button"
            onClick={onClose}
            aria-label={t("common.close")}
            className="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground hover:bg-accent"
          >
            <X className="h-4 w-4" />
          </button>
        )}
      </div>

      <Button
        type="button"
        size="md"
        className="mt-1 gap-2"
        onClick={() => handleNav("/compose")}
      >
        <PenSquare className="h-4 w-4" />
        {t("compose.new")}
      </Button>

      <nav className="mt-3 flex flex-col gap-1">
        {PRIMARY.map(({ to, labelKey, Icon }) => (
          <NavLink
            key={to}
            to={to}
            onClick={() => {
              if (isMobile) onClose?.();
            }}
            className={({ isActive }) =>
              cn(
                "flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
                isActive
                  ? "bg-accent text-accent-foreground font-medium"
                  : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
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
          onClick={() => handleNav("/onboarding")}
        >
          <Plus className="h-4 w-4" />
          {t("sidebar.add_account")}
        </button>
      </div>

      <div className="mt-2 flex flex-col gap-1 border-t border-border pt-2">
        <NavLink
          to="/inbox?activity=1"
          onClick={() => {
            if (isMobile) onClose?.();
          }}
          className="flex items-center gap-2 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent/60 hover:text-foreground"
        >
          <Sparkles className="h-4 w-4" />
          {t("sidebar.agent_activity")}
        </NavLink>
        <NavLink
          to="/settings"
          onClick={() => {
            if (isMobile) onClose?.();
          }}
          className={({ isActive }) =>
            cn(
              "flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
              isActive
                ? "bg-accent text-accent-foreground font-medium"
                : "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
            )
          }
        >
          <SettingsIcon className="h-4 w-4" />
          {t("sidebar.settings")}
        </NavLink>
      </div>
    </aside>
  );

  if (!isMobile) {
    return content;
  }

  return (
    <>
      {/* Scrim */}
      {open && (
        <div
          onClick={onClose}
          aria-hidden
          className="fixed inset-0 z-40 bg-background/60 backdrop-blur-sm animate-fade-in"
        />
      )}
      {content}
    </>
  );
}
