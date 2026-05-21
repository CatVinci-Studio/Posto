import * as React from "react";
import { useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";
import { EmailStep } from "./EmailStep";
import { PickProviderStep } from "./PickProviderStep";
import { AuthStep } from "./AuthStep";
import { TestStep } from "./TestStep";
import { SuccessStep } from "./SuccessStep";
import type { ProviderKind } from "./detect";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------
export type Step = "email" | "pick_provider" | "auth" | "test" | "success";

export interface FormState {
  email: string;
  provider: ProviderKind | null;
  // Password-based providers (iCloud app-specific password, QQ/163 auth code)
  password: string;
  // Generic IMAP fields
  imapHost: string;
  imapPort: number;
  imapEncryption: "tls" | "starttls" | "none";
  smtpHost: string;
  smtpPort: number;
  smtpEncryption: "tls" | "starttls" | "none";
  username: string;
}

const INITIAL_STATE: FormState = {
  email: "",
  provider: null,
  password: "",
  imapHost: "",
  imapPort: 993,
  imapEncryption: "tls",
  smtpHost: "",
  smtpPort: 465,
  smtpEncryption: "tls",
  username: "",
};

// ---------------------------------------------------------------------------
// Progress dots
// Logical dot steps: email(0) → auth(1) → test(2) → success(3)
// pick_provider maps to dot 0 (still in "choose account" phase)
// ---------------------------------------------------------------------------
const DOT_STEPS: Step[] = ["email", "auth", "test", "success"];

function stepToDotIndex(step: Step): number {
  if (step === "pick_provider") return 0;
  return DOT_STEPS.indexOf(step);
}

interface ProgressDotsProps {
  current: Step;
}

function ProgressDots({ current }: ProgressDotsProps) {
  const active = stepToDotIndex(current);
  return (
    <div className="flex items-center justify-center gap-2 mb-6">
      {DOT_STEPS.map((_, i) => (
        <div
          key={i}
          className={cn(
            "h-2 rounded-full transition-all duration-300",
            i === active
              ? "w-6 bg-primary"
              : i < active
              ? "w-2 bg-primary/50"
              : "w-2 bg-muted"
          )}
        />
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main Onboarding component
// ---------------------------------------------------------------------------
export function Onboarding() {
  const navigate = useNavigate();
  const [step, setStep] = React.useState<Step>("email");
  const [form, setForm] = React.useState<FormState>(INITIAL_STATE);
  // Track whether auth was reached via the picker (so Back returns to picker)
  const [cameFromPicker, setCameFromPicker] = React.useState(false);

  function update(patch: Partial<FormState>) {
    setForm((prev) => ({ ...prev, ...patch }));
  }

  // EmailStep callback: provider detected or null
  function handleEmailNext(provider: ProviderKind | null) {
    update({ provider, username: form.email });
    if (provider) {
      setCameFromPicker(false);
      setStep("auth");
    } else {
      setStep("pick_provider");
    }
  }

  // PickProviderStep callback
  function handleProviderSelect(provider: ProviderKind) {
    update({ provider });
    setCameFromPicker(true);
    setStep("auth");
  }

  function handleAuthNext() {
    setStep("test");
  }

  function handleTestSuccess() {
    setStep("success");
  }

  function handleGoToInbox() {
    navigate("/inbox");
  }

  return (
    <div className="flex h-full items-start justify-center overflow-y-auto p-6">
      <div className="w-full max-w-md py-8">
        <ProgressDots current={step} />

        {step === "email" && (
          <EmailStep
            email={form.email}
            onChange={(v) => update({ email: v })}
            onNext={handleEmailNext}
          />
        )}

        {step === "pick_provider" && (
          <PickProviderStep
            onSelect={handleProviderSelect}
            onBack={() => setStep("email")}
          />
        )}

        {step === "auth" && (
          <AuthStep
            state={form}
            onUpdate={update}
            onNext={handleAuthNext}
            onBack={() =>
              cameFromPicker ? setStep("pick_provider") : setStep("email")
            }
          />
        )}

        {step === "test" && <TestStep onSuccess={handleTestSuccess} />}

        {step === "success" && <SuccessStep onGoToInbox={handleGoToInbox} />}
      </div>
    </div>
  );
}
