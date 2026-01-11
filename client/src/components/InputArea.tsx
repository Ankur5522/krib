import { useState, useEffect } from "react";
import { Send, AlertCircle } from "lucide-react";
import { useChatStore } from "../store/useChatStore";
import { type MessageType } from "../types";
import type { Theme } from "./MessageList";

interface InputAreaProps {
  onSendMessage: (content: string, phone: string, type: MessageType) => void;
  error?: string | null;
  theme: Theme;
  darkMode: boolean;
}

const COOLDOWN_MS = 60 * 1000;

export const InputArea = ({
  onSendMessage,
  error,
  theme,
  darkMode,
}: InputAreaProps) => {
  const { activeTab, lastPostTime, markPostSent } = useChatStore();
  const [content, setContent] = useState("");
  const [phone, setPhone] = useState("");
  const [timeLeft, setTimeLeft] = useState(0);

  // Cooldown timer effect
  useEffect(() => {
    const timer = setInterval(() => {
      if (!lastPostTime) {
        setTimeLeft(0);
        return;
      }

      const diff = Date.now() - lastPostTime;
      if (diff < COOLDOWN_MS) {
        setTimeLeft(Math.ceil((COOLDOWN_MS - diff) / 1000));
      } else {
        setTimeLeft(0);
      }
    }, 1000);

    return () => clearInterval(timer);
  }, [lastPostTime]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (timeLeft > 0) return;
    if (!content.trim()) return;

    // Use default phone if not provided or enforce?
    // I'll make phone optional in UI but pass empty string if not present
    onSendMessage(content, phone, activeTab);
    markPostSent();
    setContent("");
  };

  const isOverLimit = content.length > 280;

  return (
    <div className={`fixed bottom-0 left-0 right-0 z-30`}>
      <div
        className={`${darkMode ? "bg-[#141414]" : "bg-white"} border-t ${
          theme.border
        }`}
      >
        <div className="w-full max-w-[60%] md:max-w-[60%] sm:max-w-full mx-auto px-4 py-4">
          {timeLeft > 0 && (
            <div className="mb-2 flex items-center gap-2 px-3 py-2 bg-blue-100/90 rounded-lg text-xs text-blue-700">
              <AlertCircle size={14} className="shrink-0" />
              <span>Please wait {timeLeft} seconds before posting again</span>
            </div>
          )}
          {error && (
            <div className="mb-2 flex items-center gap-2 px-3 py-2 bg-red-100/90 rounded-lg text-xs text-red-700">
              <AlertCircle size={14} className="shrink-0" />
              <span>{error}</span>
            </div>
          )}
          <form onSubmit={handleSubmit}>
            <div
              className={`flex items-center gap-2 ${
                darkMode ? "bg-[#252525]" : "bg-zinc-100"
              } rounded-full px-4 py-2`}
            >
              {/* Phone Input - Smaller Width */}
              <input
                type="tel"
                placeholder="Phone"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                className={`w-24 bg-transparent text-xs focus:outline-none ${
                  darkMode
                    ? "text-white placeholder-zinc-500"
                    : "text-black placeholder-zinc-400"
                }`}
              />
              <div
                className={`w-px h-4 ${
                  darkMode ? "bg-zinc-700" : "bg-zinc-300"
                }`}
              />
              {/* Message Input - Larger Width */}
              <textarea
                placeholder={
                  activeTab === "offered"
                    ? "Share what you're offering..."
                    : "Describe what you're looking for..."
                }
                value={content}
                onChange={(e) => setContent(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && e.shiftKey) {
                    e.preventDefault();
                    handleSubmit(e);
                  }
                }}
                maxLength={280}
                rows={1}
                className={`flex-1 bg-transparent text-sm focus:outline-none resize-none ${
                  darkMode
                    ? "text-white placeholder-zinc-500"
                    : "text-black placeholder-zinc-400"
                } py-1`}
              />
              <button
                type="submit"
                disabled={timeLeft > 0 || !content.trim() || isOverLimit}
                className={`p-2 rounded-full ${
                  theme.accent
                } transition-all hover:opacity-90 ${
                  timeLeft > 0 || !content.trim() || isOverLimit
                    ? "opacity-40"
                    : ""
                }`}
              >
                <Send className="w-4 h-4" />
              </button>
            </div>
            <div className={`text-[10px] ${theme.textMuted} mt-1 px-1`}>
              Shift+Enter to send • {content.length}/280
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};
