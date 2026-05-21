import * as React from "react";
import { cn } from "@/lib/utils";
import { call } from "@/lib/tauri";
import type { Account } from "@/types/email";

interface AccountSelectProps {
  value?: number;
  onChange: (accountId: number) => void;
  className?: string;
}

const MOCK_ACCOUNTS: Account[] = [
  {
    id: 1,
    provider: "gmail",
    email: "you@example.com",
    display_name: "You",
    status: "active",
  },
];

/**
 * Account selector dropdown for the From field.
 * Fetches connected accounts; falls back to mock data if backend unavailable.
 */
export function AccountSelect({ value, onChange, className }: AccountSelectProps) {
  const [accounts, setAccounts] = React.useState<Account[]>(MOCK_ACCOUNTS);

  React.useEffect(() => {
    call<Account[]>("list_accounts")
      .then((accs) => {
        if (accs && accs.length > 0) setAccounts(accs);
      })
      .catch(() => {
        // Backend not ready — keep mock fallback
      });
  }, []);

  const selectedId = value ?? accounts[0]?.id;

  React.useEffect(() => {
    if (!value && accounts.length > 0) {
      onChange(accounts[0].id);
    }
  }, [accounts, value, onChange]);

  return (
    <div className={cn("relative", className)}>
      <select
        value={selectedId ?? ""}
        onChange={(e) => onChange(Number(e.target.value))}
        className={cn(
          "h-8 appearance-none rounded-md border border-border bg-background pl-3 pr-7 text-sm text-foreground",
          "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
        )}
      >
        {accounts.map((acc) => (
          <option key={acc.id} value={acc.id}>
            {acc.display_name ? `${acc.display_name} <${acc.email}>` : acc.email}
          </option>
        ))}
      </select>
      {/* Chevron */}
      <svg
        className="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="m6 9 6 6 6-6" />
      </svg>
    </div>
  );
}
