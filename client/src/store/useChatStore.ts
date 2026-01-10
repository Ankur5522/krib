import { create } from "zustand";
import { type Message, type MessageType } from "../types";

interface ChatState {
  messages: Message[];
  activeTab: MessageType;
  lastPostTime: number; // For cooldown calculation

  setTab: (tab: MessageType) => void;
  addMessage: (msg: Message) => void;
  clearMessages: () => void;
  markPostSent: () => void;
  setCooldown: (seconds: number) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  messages: [],
  activeTab: "offered",
  lastPostTime: 0,

  setTab: (tab) => set({ activeTab: tab }),

  addMessage: (msg) =>
    set((state) => {
      // Prevent duplicates if any
      if (state.messages.some((m) => m.id === msg.id)) return state;
      return { messages: [...state.messages, msg] };
    }),

  clearMessages: () => set({ messages: [] }),

  markPostSent: () => set({ lastPostTime: Date.now() }),

  setCooldown: (seconds) => {
    // Set lastPostTime such that the remaining cooldown equals the given seconds
    // lastPostTime should be: now - (60000ms - remaining_ms)
    const remainingMs = seconds * 1000;
    const cooldownMs = 60 * 1000;
    const adjustedTime = Date.now() - (cooldownMs - remainingMs);
    set({ lastPostTime: adjustedTime });
  },
}));
