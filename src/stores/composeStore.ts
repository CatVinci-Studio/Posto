import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { DraftForm } from "@/types/compose";

export type { DraftForm };

interface ComposeState {
  drafts: Record<string, DraftForm>;
  saveDraft: (key: string, form: DraftForm) => void;
  loadDraft: (key: string) => DraftForm | undefined;
  clearDraft: (key: string) => void;
}

export const useComposeStore = create<ComposeState>()(
  persist(
    (set, get) => ({
      drafts: {},

      saveDraft: (key, form) =>
        set((state) => ({
          drafts: { ...state.drafts, [key]: form },
        })),

      loadDraft: (key) => get().drafts[key],

      clearDraft: (key) =>
        set((state) => {
          const next = { ...state.drafts };
          delete next[key];
          return { drafts: next };
        }),
    }),
    {
      name: "retposto.compose.drafts",
    }
  )
);
