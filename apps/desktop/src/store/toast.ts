import { create } from "zustand";

interface ToastState {
  message: string | null;
  tone: "info" | "error";
  show: (message: string, tone?: "info" | "error") => void;
  clear: () => void;
}

let timer: ReturnType<typeof setTimeout> | undefined;

export const useToastStore = create<ToastState>((set) => ({
  message: null,
  tone: "info",
  show: (message, tone = "info") => {
    if (timer) clearTimeout(timer);
    set({ message, tone });
    timer = setTimeout(() => set({ message: null }), 3200);
  },
  clear: () => {
    if (timer) clearTimeout(timer);
    set({ message: null });
  },
}));

export function toast(message: string) {
  useToastStore.getState().show(message);
}

export function toastError(message: string) {
  useToastStore.getState().show(message, "error");
}
