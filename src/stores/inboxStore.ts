import { create } from "zustand";
import type { AgentCategory } from "@/types/email";

export interface InboxFilter {
  account_id?: number;
  category?: AgentCategory;
  status?: "today" | "week" | "newsletters" | "done";
}

interface InboxState {
  selectedMessageId: number | null;
  filter: InboxFilter;
  showActivityDrawer: boolean;
  setSelected: (id: number | null) => void;
  setFilter: (filter: InboxFilter) => void;
  toggleActivityDrawer: () => void;
}

export const useInboxStore = create<InboxState>((set) => ({
  selectedMessageId: null,
  filter: {},
  showActivityDrawer: false,
  setSelected: (id) => set({ selectedMessageId: id }),
  setFilter: (filter) => set({ filter }),
  toggleActivityDrawer: () =>
    set((state) => ({ showActivityDrawer: !state.showActivityDrawer })),
}));
