import * as React from "react";
import { useTranslation } from "react-i18next";
import { Eye, EyeOff, CheckCircle2, AlertCircle, Shield } from "lucide-react";
import { Card, CardHeader, CardContent, CardFooter } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { useSettingsStore } from "@/stores/settingsStore";
import { call } from "@/lib/tauri";

const OPENAI_MODELS = [
  { value: "gpt-4o-mini", label: "GPT-4o mini (recommended)" },
  { value: "gpt-4o", label: "GPT-4o" },
  { value: "gpt-4-turbo", label: "GPT-4 Turbo" },
  { value: "gpt-3.5-turbo", label: "GPT-3.5 Turbo" },
];

type TestState = "idle" | "testing" | "ok" | "fail";

export function LlmTab() {
  const { t } = useTranslation();
  const { llm_model, setLlmModel } = useSettingsStore();

  const [apiKey, setApiKey] = React.useState("");
  const [showKey, setShowKey] = React.useState(false);
  const [keyConfigured, setKeyConfigured] = React.useState(false);
  const [testState, setTestState] = React.useState<TestState>("idle");
  const [testMessage, setTestMessage] = React.useState("");
  const [confirmClear, setConfirmClear] = React.useState(false);
  const [comingSoonVisible, setComingSoonVisible] = React.useState(false);

  // Check initial key status
  React.useEffect(() => {
    call<boolean>("openai_key_configured")
      .then(setKeyConfigured)
      .catch(() => setKeyConfigured(false));
  }, []);

  const [saveError, setSaveError] = React.useState<string | null>(null);
  async function handleSaveKey() {
    setSaveError(null);
    try {
      await call("set_openai_api_key", { key: apiKey });
      setKeyConfigured(true);
      setApiKey("");
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleTest() {
    setTestState("testing");
    setTestMessage("");
    try {
      const result = await call<string>("test_openai_completion", {
        prompt: "Reply with just OK.",
      });
      setTestState("ok");
      setTestMessage(result || "OK");
    } catch (err) {
      setTestState("fail");
      setTestMessage(err instanceof Error ? err.message : "Connection failed");
    }
  }

  async function handleClearKey() {
    try {
      await call("clear_openai_api_key");
    } catch {
      // fallback
    }
    setKeyConfigured(false);
    setConfirmClear(false);
  }

  return (
    <div className="space-y-6">
      {/* Provider header */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-base font-semibold">{t("settings.tabs.llm")}</h2>
              <p className="text-xs text-muted-foreground">OpenAI</p>
            </div>
            <span className="inline-flex items-center rounded-full border border-amber-300 bg-amber-50 dark:bg-amber-950 px-2.5 py-0.5 text-xs text-amber-700 dark:text-amber-300 font-medium">
              Experimental: ChatGPT OAuth
            </span>
          </div>
        </CardHeader>

        {/* API Key section */}
        <CardContent className="space-y-4">
          <div className="space-y-1.5">
            <label className="text-sm font-medium">{t("settings.llm.api_key")}</label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <Input
                  type={showKey ? "text" : "password"}
                  placeholder={t("settings.llm.api_key_placeholder")}
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  className="pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowKey((v) => !v)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                >
                  {showKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
              <Button
                variant="default"
                size="md"
                onClick={handleSaveKey}
                disabled={!apiKey.trim()}
              >
                {t("settings.llm.save")}
              </Button>
            </div>

            {saveError && (
              <p className="mt-1 text-xs text-destructive">{saveError}</p>
            )}

            {/* Status indicator */}
            <div className="flex items-center justify-between mt-1">
              {keyConfigured ? (
                <span className="flex items-center gap-1.5 text-xs text-green-600 dark:text-green-400">
                  <CheckCircle2 className="h-3.5 w-3.5" />
                  {t("settings.llm.key_configured")}
                </span>
              ) : (
                <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  <AlertCircle className="h-3.5 w-3.5" />
                  {t("settings.llm.no_key")}
                </span>
              )}
              {keyConfigured && !confirmClear && (
                <button
                  className="text-xs text-red-500 hover:underline"
                  onClick={() => setConfirmClear(true)}
                >
                  {t("settings.llm.clear_key")}
                </button>
              )}
              {confirmClear && (
                <div className="flex items-center gap-2 text-xs">
                  <span className="text-muted-foreground">{t("settings.llm.confirm_clear")}</span>
                  <button
                    className="text-red-600 font-medium hover:underline"
                    onClick={handleClearKey}
                  >
                    Yes, clear
                  </button>
                  <button
                    className="text-muted-foreground hover:underline"
                    onClick={() => setConfirmClear(false)}
                  >
                    Cancel
                  </button>
                </div>
              )}
            </div>
          </div>

          {/* Test connection */}
          <div className="space-y-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleTest}
              disabled={testState === "testing"}
              className="gap-2"
            >
              {testState === "testing" && (
                <svg className="h-3.5 w-3.5 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
                </svg>
              )}
              {testState === "testing"
                ? t("settings.llm.test_in_progress")
                : t("settings.llm.test_connection")}
            </Button>
            {testState === "ok" && (
              <p className="flex items-center gap-1 text-xs text-green-600 dark:text-green-400">
                <CheckCircle2 className="h-3.5 w-3.5" />
                {t("settings.llm.test_success")}: {testMessage}
              </p>
            )}
            {testState === "fail" && (
              <p className="flex items-center gap-1 text-xs text-red-500">
                <AlertCircle className="h-3.5 w-3.5" />
                {t("settings.llm.test_failure")}: {testMessage}
              </p>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Model picker */}
      <Card>
        <CardHeader>
          <h2 className="text-base font-semibold">{t("settings.llm.model")}</h2>
          <p className="text-xs text-muted-foreground">Select the OpenAI model to use for agent tasks</p>
        </CardHeader>
        <CardContent>
          <Select
            value={llm_model}
            onChange={(e) => setLlmModel(e.target.value)}
            className="max-w-xs"
          >
            {OPENAI_MODELS.map((m) => (
              <option key={m.value} value={m.value}>
                {m.label}
              </option>
            ))}
          </Select>
        </CardContent>
      </Card>

      {/* ChatGPT OAuth (experimental) */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <h2 className="text-base font-semibold">ChatGPT Sign-in</h2>
            <span className="inline-flex items-center rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
              Experimental
            </span>
          </div>
          <p className="text-xs text-muted-foreground">
            Use your ChatGPT subscription instead of a separate API key
          </p>
        </CardHeader>
        <CardFooter className="bg-muted/50 rounded-b-xl">
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              setComingSoonVisible(true);
              setTimeout(() => setComingSoonVisible(false), 2500);
            }}
          >
            Sign in with ChatGPT
          </Button>
          {comingSoonVisible && (
            <span className="text-xs text-muted-foreground ml-2">Coming soon</span>
          )}
        </CardFooter>
      </Card>

      {/* Privacy note */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <Shield className="h-4 w-4 text-primary" />
            <h2 className="text-base font-semibold">Privacy</h2>
          </div>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">{t("settings.llm.privacy_note")}</p>
        </CardContent>
      </Card>
    </div>
  );
}
