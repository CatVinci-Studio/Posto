import * as React from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import {
  Plus,
  RefreshCw,
  MoreHorizontal,
  Pencil,
  Trash2,
  AlertCircle,
  CheckCircle2,
  Loader2,
} from "lucide-react";
import { Card, CardHeader, CardContent } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { Dialog, DialogFooter } from "@/components/ui/Dialog";
import { call } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import { formatRelativeTime } from "@/lib/utils";
import type { AccountSummary, ProviderKind } from "@/types/settings";

// ---------------------------------------------------------------------------
// Provider color/label helpers
// ---------------------------------------------------------------------------
const PROVIDER_COLORS: Record<ProviderKind, string> = {
  gmail: "bg-red-500",
  outlook: "bg-blue-500",
  icloud: "bg-gray-700",
  qq: "bg-sky-600",
  mail163: "bg-orange-500",
  generic_imap: "bg-violet-500",
};

const PROVIDER_INITIALS: Record<ProviderKind, string> = {
  gmail: "G",
  outlook: "O",
  icloud: "iC",
  qq: "QQ",
  mail163: "163",
  generic_imap: "IM",
};

function StatusBadge({ status }: { status: AccountSummary["status"] }) {
  if (status === "active") {
    return (
      <span className="flex items-center gap-1 text-xs text-green-600 dark:text-green-400">
        <CheckCircle2 className="h-3.5 w-3.5" />
        Active
      </span>
    );
  }
  if (status === "syncing") {
    return (
      <span className="flex items-center gap-1 text-xs text-amber-500">
        <Loader2 className="h-3.5 w-3.5 animate-spin" />
        Syncing
      </span>
    );
  }
  return (
    <span className="flex items-center gap-1 text-xs text-red-500">
      <AlertCircle className="h-3.5 w-3.5" />
      Error
    </span>
  );
}

// ---------------------------------------------------------------------------
// Account row
// ---------------------------------------------------------------------------
interface AccountRowProps {
  account: AccountSummary;
  onRemove: (id: number) => void;
  onSync: (id: number) => void;
}

function AccountRow({ account, onRemove, onSync }: AccountRowProps) {
  const { t } = useTranslation();
  const [menuOpen, setMenuOpen] = React.useState(false);
  const menuRef = React.useRef<HTMLDivElement>(null);

  // Close menu on outside click
  React.useEffect(() => {
    function handle(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setMenuOpen(false);
      }
    }
    if (menuOpen) document.addEventListener("mousedown", handle);
    return () => document.removeEventListener("mousedown", handle);
  }, [menuOpen]);

  return (
    <div className="flex items-center gap-4 py-3 px-4 rounded-lg hover:bg-muted/50 transition-colors">
      {/* Avatar */}
      <div
        className={cn(
          "flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-full text-white text-xs font-bold",
          PROVIDER_COLORS[account.provider]
        )}
      >
        {PROVIDER_INITIALS[account.provider]}
      </div>

      {/* Info */}
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium truncate">
          {account.display_name ?? account.email}
        </p>
        <p className="text-xs text-muted-foreground truncate">{account.email}</p>
        {account.status === "error" && account.error_message && (
          <p className="text-xs text-red-500 truncate">{account.error_message}</p>
        )}
      </div>

      {/* Status + last synced */}
      <div className="flex flex-col items-end gap-0.5 text-right shrink-0">
        <StatusBadge status={account.status} />
        {account.last_sync_at && (
          <span className="text-xs text-muted-foreground">
            {t("settings.accounts.last_synced")}: {formatRelativeTime(account.last_sync_at)}
          </span>
        )}
      </div>

      {/* Actions menu */}
      <div className="relative shrink-0" ref={menuRef}>
        <Button
          variant="ghost"
          size="sm"
          className="h-8 w-8 p-0"
          onClick={() => setMenuOpen((v) => !v)}
        >
          <MoreHorizontal className="h-4 w-4" />
        </Button>
        {menuOpen && (
          <div className="absolute right-0 top-9 z-50 min-w-[140px] rounded-md border border-border bg-background shadow-md py-1">
            <button
              className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-muted"
              onClick={() => { onSync(account.id); setMenuOpen(false); }}
            >
              <RefreshCw className="h-4 w-4" />
              {t("settings.accounts.actions.sync_now")}
            </button>
            <button
              className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-muted"
              onClick={() => setMenuOpen(false)}
            >
              <Pencil className="h-4 w-4" />
              {t("settings.accounts.actions.edit")}
            </button>
            <button
              className="flex w-full items-center gap-2 px-3 py-2 text-sm text-red-600 hover:bg-muted"
              onClick={() => { onRemove(account.id); setMenuOpen(false); }}
            >
              <Trash2 className="h-4 w-4" />
              {t("settings.accounts.actions.remove")}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}


// ---------------------------------------------------------------------------
// AccountsTab
// ---------------------------------------------------------------------------
export function AccountsTab() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [accounts, setAccounts] = React.useState<AccountSummary[]>([]);
  const [confirmId, setConfirmId] = React.useState<number | null>(null);

  const [loadError, setLoadError] = React.useState<string | null>(null);

  const reload = React.useCallback(() => {
    call<AccountSummary[]>("list_accounts")
      .then((data) => {
        setAccounts(data);
        setLoadError(null);
      })
      .catch((err) => {
        setAccounts([]);
        setLoadError(err instanceof Error ? err.message : String(err));
      });
  }, []);

  React.useEffect(() => {
    reload();
  }, [reload]);

  function handleSync(id: number) {
    call("trigger_sync", { account_id: id }).then(reload).catch(() => {});
    setAccounts((prev) =>
      prev.map((a) => (a.id === id ? { ...a, status: "syncing" } : a))
    );
  }

  function handleRemoveRequest(id: number) {
    setConfirmId(id);
  }

  function handleRemoveConfirm() {
    if (confirmId == null) return;
    call("remove_account", { id: confirmId }).catch(() => {});
    setAccounts((prev) => prev.filter((a) => a.id !== confirmId));
    setConfirmId(null);
  }

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-base font-semibold">{t("settings.tabs.accounts")}</h2>
              <p className="text-xs text-muted-foreground">
                {accounts.length} account{accounts.length !== 1 ? "s" : ""} connected
              </p>
            </div>
            <Button
              variant="default"
              size="sm"
              onClick={() => navigate("/onboarding")}
              className="gap-1.5"
            >
              <Plus className="h-4 w-4" />
              {t("settings.accounts.add")}
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {loadError && (
            <div className="mb-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive">
              {loadError}
            </div>
          )}
          {accounts.length === 0 && !loadError ? (
            <p className="text-sm text-muted-foreground py-4 text-center">
              {t("settings.accounts.no_accounts")}
            </p>
          ) : (
            <div className="divide-y divide-border -mx-2">
              {accounts.map((account) => (
                <AccountRow
                  key={account.id}
                  account={account}
                  onRemove={handleRemoveRequest}
                  onSync={handleSync}
                />
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog
        open={confirmId != null}
        onClose={() => setConfirmId(null)}
        title="Remove Account"
      >
        <p className="text-sm text-foreground">{t("settings.accounts.confirm_remove")}</p>
        <DialogFooter>
          <Button variant="outline" size="sm" onClick={() => setConfirmId(null)}>
            Cancel
          </Button>
          <Button
            size="sm"
            className="bg-red-600 hover:bg-red-700 text-white"
            onClick={handleRemoveConfirm}
          >
            Remove
          </Button>
        </DialogFooter>
      </Dialog>
    </div>
  );
}
