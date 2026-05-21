import { create } from "zustand";
import { persist } from "zustand/middleware";
import { call } from "@/lib/tauri";
import { setUiLanguage } from "@/i18n";
import type { AppSettings, UiLang, DisplayLang, ReplyLang, TrustLevel } from "@/types/settings";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
function detectUiLang(): UiLang {
  if (typeof navigator !== "undefined" && navigator.language.startsWith("zh")) {
    return "zh";
  }
  return "en";
}

function applyTheme(theme: AppSettings["theme"]) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.classList.remove("dark", "light");
  if (theme === "dark") {
    root.classList.add("dark");
  } else if (theme === "light") {
    root.classList.add("light");
  } else {
    // system
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.classList.add(prefersDark ? "dark" : "light");
  }
}

const DEFAULT_SETTINGS: AppSettings = {
  ui_lang: detectUiLang(),
  display_lang: "original",
  reply_lang: "auto",
  trust_level: "medium",
  theme: "system",
  llm_provider: "openai",
  llm_model: "gpt-4o-mini",
  per_action: {
    classify: "auto",
    label: "auto",
    archive_newsletter: "auto",
    move_folder: "auto",
    mark_read: "auto",
    generate_draft: "auto",
  },
};

// ---------------------------------------------------------------------------
// Store interface
// ---------------------------------------------------------------------------
interface SettingsState extends AppSettings {
  hydrated: boolean;
  hydrateFromBackend: () => Promise<void>;

  setUiLang: (lang: UiLang) => void;
  setDisplayLang: (lang: DisplayLang) => void;
  setReplyLang: (lang: ReplyLang) => void;
  setTrustLevel: (level: TrustLevel) => void;
  setTheme: (theme: AppSettings["theme"]) => void;
  setLlmModel: (model: string) => void;
  setPerAction: (
    action: keyof AppSettings["per_action"],
    value: "auto" | "ask"
  ) => void;
  updateSettings: (patch: Partial<AppSettings>) => void;
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------
export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      ...DEFAULT_SETTINGS,
      hydrated: false,

      hydrateFromBackend: async () => {
        try {
          const remote = await call<AppSettings>("get_settings");
          set({ ...remote, hydrated: true });
          setUiLanguage(remote.ui_lang);
          applyTheme(remote.theme);
        } catch {
          // Backend not available — fall back to persisted / default values
          const current = get();
          setUiLanguage(current.ui_lang);
          applyTheme(current.theme);
          set({ hydrated: true });
        }
      },

      setUiLang: (lang) => {
        set({ ui_lang: lang });
        setUiLanguage(lang);
      },

      setDisplayLang: (lang) => set({ display_lang: lang }),

      setReplyLang: (lang) => set({ reply_lang: lang }),

      setTrustLevel: (level) => set({ trust_level: level }),

      setTheme: (theme) => {
        set({ theme });
        applyTheme(theme);
      },

      setLlmModel: (model) => set({ llm_model: model }),

      setPerAction: (action, value) =>
        set((s) => ({
          per_action: { ...s.per_action, [action]: value },
        })),

      updateSettings: (patch) => {
        set(patch);
        if (patch.ui_lang) setUiLanguage(patch.ui_lang);
        if (patch.theme) applyTheme(patch.theme);
      },
    }),
    {
      name: "posto.settings",
      // Persist all settings fields except hydrated
      partialize: (state) => {
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        const { hydrated, hydrateFromBackend, setUiLang, setDisplayLang, setReplyLang, setTrustLevel, setTheme, setLlmModel, setPerAction, updateSettings, ...rest } = state;
        return rest;
      },
    }
  )
);
