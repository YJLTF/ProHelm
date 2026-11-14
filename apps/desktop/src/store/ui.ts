import { create } from "zustand";

interface UiState {
  sidebarCollapsed: boolean;
  copilotOpen: boolean;
  toggleSidebar: () => void;
  setCopilotOpen: (open: boolean) => void;
}

export const useUiStore = create<UiState>((set) => ({
  sidebarCollapsed: false,
  copilotOpen: false,
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  setCopilotOpen: (open) => set({ copilotOpen: open }),
}));
