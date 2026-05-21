export type ProviderKind =
  | "gmail"
  | "outlook"
  | "icloud"
  | "qq"
  | "mail163"
  | "generic_imap";

const DOMAIN_MAP: Record<string, ProviderKind> = {
  // Gmail
  "gmail.com": "gmail",
  "googlemail.com": "gmail",
  // Outlook
  "outlook.com": "outlook",
  "hotmail.com": "outlook",
  "live.com": "outlook",
  "msn.com": "outlook",
  // iCloud
  "icloud.com": "icloud",
  "me.com": "icloud",
  "mac.com": "icloud",
  // QQ
  "qq.com": "qq",
  "foxmail.com": "qq",
  "vip.qq.com": "qq",
  // 163
  "163.com": "mail163",
  "126.com": "mail163",
  "yeah.net": "mail163",
};

/**
 * Detect provider from email address.
 * Returns null if no known provider matches (→ show provider picker).
 */
export function detectProvider(email: string): ProviderKind | null {
  const atIndex = email.lastIndexOf("@");
  if (atIndex === -1) return null;
  const domain = email.slice(atIndex + 1).toLowerCase().trim();
  return DOMAIN_MAP[domain] ?? null;
}
