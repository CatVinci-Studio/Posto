import * as React from "react";
import { useTranslation } from "react-i18next";
import { LogIn, Eye, EyeOff, CheckCircle2, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/Card";
import { cn } from "@/lib/utils";
import type { ProviderKind } from "./detect";
import type { FormState } from "./index";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

// ---------------------------------------------------------------------------
// OAuth sub-component (Gmail / Outlook)
// ---------------------------------------------------------------------------
interface OAuthAuthProps {
  provider: "gmail" | "outlook";
  onSuccess: () => void;
  onBack: () => void;
}

function OAuthAuth({ provider, onSuccess, onBack }: OAuthAuthProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = React.useState(false);

  async function handleSignIn() {
    setLoading(true);
    // TODO: replace with real IPC call to begin_oauth_login and listen for oauth:callback event
    await sleep(800);
    setLoading(false);
    onSuccess();
  }

  const providerName = t(`onboarding.providers.${provider}`);

  return (
    <Card className="w-full">
      <CardHeader>
        <h1 className="text-2xl font-semibold">
          {t("onboarding.signin_with", { provider: providerName })}
        </h1>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <Button size="lg" className="w-full" onClick={handleSignIn} disabled={loading}>
          <LogIn className="h-4 w-4" />
          {loading
            ? t("onboarding.testing_connection")
            : t("onboarding.signin_with", { provider: providerName })}
        </Button>
        <p className="text-sm text-muted-foreground text-center">
          {t("onboarding.oauth_redirect_explainer", { provider: providerName })}
        </p>
      </CardContent>
      <CardFooter>
        <Button variant="ghost" size="sm" onClick={onBack} className="w-full">
          {t("onboarding.back")}
        </Button>
      </CardFooter>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// App-specific password sub-component (iCloud)
// ---------------------------------------------------------------------------
interface AppPasswordAuthProps {
  email: string;
  password: string;
  username: string;
  onPasswordChange: (v: string) => void;
  onUsernameChange: (v: string) => void;
  onNext: () => void;
  onBack: () => void;
}

function AppPasswordAuth({
  email,
  password,
  username,
  onPasswordChange,
  onUsernameChange,
  onNext,
  onBack,
}: AppPasswordAuthProps) {
  const { t } = useTranslation();
  const [showPw, setShowPw] = React.useState(false);

  function openAppleId() {
    // TODO: call('open_external', { url: 'https://account.apple.com/account/manage' })
    window.open("https://account.apple.com/account/manage", "_blank");
  }

  const guideSteps: string[] = t("onboarding.guide.icloud", {
    returnObjects: true,
  }) as string[];

  return (
    <Card className="w-full">
      <CardHeader>
        <h1 className="text-2xl font-semibold">
          {t("onboarding.providers.icloud")}
        </h1>
      </CardHeader>
      <CardContent className="flex flex-col gap-5">
        <ol className="list-decimal list-inside space-y-1.5 text-sm text-muted-foreground">
          {guideSteps.map((step, i) => (
            <li key={i}>{step}</li>
          ))}
        </ol>
        <Button variant="outline" size="sm" onClick={openAppleId} className="w-full gap-2">
          <ExternalLink className="h-4 w-4" />
          {t("onboarding.open_settings_page")}
        </Button>
        <div className="relative">
          <Input
            label={t("onboarding.app_password_label")}
            type={showPw ? "text" : "password"}
            value={password}
            onChange={(e) => onPasswordChange(e.target.value)}
            autoComplete="current-password"
          />
          <button
            type="button"
            onClick={() => setShowPw((v) => !v)}
            className="absolute right-3 top-8 text-muted-foreground hover:text-foreground"
            tabIndex={-1}
          >
            {showPw ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </button>
        </div>
        <Input
          label={t("onboarding.username_label")}
          type="text"
          value={username || email}
          onChange={(e) => onUsernameChange(e.target.value)}
          autoComplete="username"
        />
      </CardContent>
      <CardFooter className="flex-col items-stretch gap-2">
        <Button size="lg" className="w-full" onClick={onNext} disabled={!password}>
          {t("onboarding.next")}
        </Button>
        <Button variant="ghost" size="sm" onClick={onBack} className="w-full">
          {t("onboarding.back")}
        </Button>
      </CardFooter>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Auth-code sub-component (QQ / 163)
// ---------------------------------------------------------------------------
interface AuthCodeAuthProps {
  provider: "qq" | "mail163";
  email: string;
  password: string;
  username: string;
  onPasswordChange: (v: string) => void;
  onUsernameChange: (v: string) => void;
  onNext: () => void;
  onBack: () => void;
}

const PROVIDER_URLS: Record<string, string> = {
  qq: "https://wx.mail.qq.com/account",
  mail163: "https://mail.163.com/account/manage",
};

function AuthCodeAuth({
  provider,
  email,
  password,
  username,
  onPasswordChange,
  onUsernameChange,
  onNext,
  onBack,
}: AuthCodeAuthProps) {
  const { t } = useTranslation();
  const [codeValid, setCodeValid] = React.useState(false);

  const guideKey = provider === "qq" ? "onboarding.guide.qq" : "onboarding.guide.163";
  const guideSteps: string[] = t(guideKey, { returnObjects: true }) as string[];
  const providerName = t(`onboarding.providers.${provider}`);

  function handleCodeChange(raw: string) {
    // Auto-trim whitespace on paste / input
    const trimmed = raw.replace(/\s/g, "");
    onPasswordChange(trimmed);
    setCodeValid(/^[A-Za-z0-9]{16}$/.test(trimmed));
  }

  function openSettings() {
    // TODO: call('open_external', { url: PROVIDER_URLS[provider] })
    window.open(PROVIDER_URLS[provider], "_blank");
  }

  return (
    <Card className="w-full">
      <CardHeader>
        <h1 className="text-2xl font-semibold">{providerName}</h1>
      </CardHeader>
      <CardContent className="flex flex-col gap-5">
        <ol className="list-decimal list-inside space-y-1.5 text-sm text-muted-foreground">
          {guideSteps.map((step, i) => (
            <li key={i}>{step}</li>
          ))}
        </ol>
        <Button variant="outline" size="sm" onClick={openSettings} className="w-full gap-2">
          <ExternalLink className="h-4 w-4" />
          {t("onboarding.open_settings_page")}
        </Button>
        <div className="relative">
          <Input
            label={t("onboarding.auth_code_label")}
            type="text"
            value={password}
            onChange={(e) => handleCodeChange(e.target.value)}
            onPaste={(e) => {
              e.preventDefault();
              const pasted = e.clipboardData.getData("text");
              handleCodeChange(pasted);
            }}
            placeholder="16-character code"
            maxLength={16}
          />
          {codeValid && (
            <CheckCircle2 className="absolute right-3 top-8 h-4 w-4 text-green-500" />
          )}
        </div>
        <Input
          label={t("onboarding.username_label")}
          type="text"
          value={username || email}
          onChange={(e) => onUsernameChange(e.target.value)}
          autoComplete="username"
        />
      </CardContent>
      <CardFooter className="flex-col items-stretch gap-2">
        <Button size="lg" className="w-full" onClick={onNext} disabled={!codeValid}>
          {t("onboarding.next")}
        </Button>
        <Button variant="ghost" size="sm" onClick={onBack} className="w-full">
          {t("onboarding.back")}
        </Button>
      </CardFooter>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Generic IMAP sub-component
// ---------------------------------------------------------------------------
type Encryption = "tls" | "starttls" | "none";

interface GenericImapAuthProps {
  form: FormState;
  onUpdate: (patch: Partial<FormState>) => void;
  onNext: () => void;
  onBack: () => void;
}

function GenericImapAuth({ form, onUpdate, onNext, onBack }: GenericImapAuthProps) {
  const { t } = useTranslation();
  const [showPw, setShowPw] = React.useState(false);

  function handleImapHostChange(host: string) {
    const smtpHost = host.replace(/^imap\./i, "smtp.");
    onUpdate({ imapHost: host, smtpHost });
  }

  const encryptionOptions: { value: Encryption; label: string }[] = [
    { value: "tls",      label: t("onboarding.encryption.tls") },
    { value: "starttls", label: t("onboarding.encryption.starttls") },
    { value: "none",     label: t("onboarding.encryption.none") },
  ];

  function RadioGroup({
    name,
    value,
    onChange,
  }: {
    name: string;
    value: Encryption;
    onChange: (v: Encryption) => void;
  }) {
    return (
      <div className="flex gap-4">
        {encryptionOptions.map((opt) => (
          <label key={opt.value} className="flex items-center gap-1.5 text-sm cursor-pointer">
            <input
              type="radio"
              name={name}
              value={opt.value}
              checked={value === opt.value}
              onChange={() => onChange(opt.value)}
              className="accent-primary"
            />
            {opt.label}
          </label>
        ))}
      </div>
    );
  }

  const isValid =
    form.imapHost.trim() &&
    form.smtpHost.trim() &&
    form.username.trim() &&
    form.password.trim();

  return (
    <Card className="w-full">
      <CardHeader>
        <h1 className="text-2xl font-semibold">{t("onboarding.providers.generic_imap")}</h1>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <div className="rounded-md border border-border p-4 space-y-3">
          <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            IMAP
          </p>
          <Input
            label={t("onboarding.imap_host")}
            type="text"
            value={form.imapHost}
            onChange={(e) => handleImapHostChange(e.target.value)}
            placeholder="imap.example.com"
          />
          <div className="grid grid-cols-2 gap-3">
            <Input
              label={t("onboarding.imap_port")}
              type="number"
              value={form.imapPort}
              onChange={(e) => onUpdate({ imapPort: Number(e.target.value) })}
            />
            <div className="flex flex-col gap-1.5">
              <span className="text-sm font-medium">{t("onboarding.imap_encryption")}</span>
              <RadioGroup
                name="imapEnc"
                value={form.imapEncryption}
                onChange={(v) => onUpdate({ imapEncryption: v })}
              />
            </div>
          </div>
        </div>

        <div className="rounded-md border border-border p-4 space-y-3">
          <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            SMTP
          </p>
          <Input
            label={t("onboarding.smtp_host")}
            type="text"
            value={form.smtpHost}
            onChange={(e) => onUpdate({ smtpHost: e.target.value })}
            placeholder="smtp.example.com"
          />
          <div className="grid grid-cols-2 gap-3">
            <Input
              label={t("onboarding.smtp_port")}
              type="number"
              value={form.smtpPort}
              onChange={(e) => onUpdate({ smtpPort: Number(e.target.value) })}
            />
            <div className="flex flex-col gap-1.5">
              <span className="text-sm font-medium">{t("onboarding.smtp_encryption")}</span>
              <RadioGroup
                name="smtpEnc"
                value={form.smtpEncryption}
                onChange={(v) => onUpdate({ smtpEncryption: v })}
              />
            </div>
          </div>
        </div>

        <Input
          label={t("onboarding.username_label")}
          type="text"
          value={form.username}
          onChange={(e) => onUpdate({ username: e.target.value })}
          autoComplete="username"
        />
        <div className="relative">
          <Input
            label={t("onboarding.password_label")}
            type={showPw ? "text" : "password"}
            value={form.password}
            onChange={(e) => onUpdate({ password: e.target.value })}
            autoComplete="current-password"
          />
          <button
            type="button"
            onClick={() => setShowPw((v) => !v)}
            className="absolute right-3 top-8 text-muted-foreground hover:text-foreground"
            tabIndex={-1}
          >
            {showPw ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </button>
        </div>
      </CardContent>
      <CardFooter className="flex-col items-stretch gap-2">
        <Button size="lg" className="w-full" onClick={onNext} disabled={!isValid}>
          {t("onboarding.next")}
        </Button>
        <Button variant="ghost" size="sm" onClick={onBack} className="w-full">
          {t("onboarding.back")}
        </Button>
      </CardFooter>
    </Card>
  );
}

// ---------------------------------------------------------------------------
// Top-level AuthStep dispatcher
// ---------------------------------------------------------------------------
interface AuthStepProps {
  state: FormState;
  onUpdate: (patch: Partial<FormState>) => void;
  onNext: () => void;
  onBack: () => void;
}

export function AuthStep({ state, onUpdate, onNext, onBack }: AuthStepProps) {
  const provider = state.provider as ProviderKind;

  if (provider === "gmail" || provider === "outlook") {
    return (
      <OAuthAuth
        provider={provider}
        onSuccess={onNext}
        onBack={onBack}
      />
    );
  }

  if (provider === "icloud") {
    return (
      <AppPasswordAuth
        email={state.email}
        password={state.password}
        username={state.username}
        onPasswordChange={(v) => onUpdate({ password: v })}
        onUsernameChange={(v) => onUpdate({ username: v })}
        onNext={onNext}
        onBack={onBack}
      />
    );
  }

  if (provider === "qq" || provider === "mail163") {
    return (
      <AuthCodeAuth
        provider={provider}
        email={state.email}
        password={state.password}
        username={state.username}
        onPasswordChange={(v) => onUpdate({ password: v })}
        onUsernameChange={(v) => onUpdate({ username: v })}
        onNext={onNext}
        onBack={onBack}
      />
    );
  }

  // generic_imap fallback
  return (
    <GenericImapAuth
      form={state}
      onUpdate={onUpdate}
      onNext={onNext}
      onBack={onBack}
    />
  );
}
