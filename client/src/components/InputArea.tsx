import { useState, useEffect, useRef } from "react";
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
const MAX_ROWS = 3;

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
  const [rows, setRows] = useState(1);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

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

    onSendMessage(content, phone, activeTab);
    markPostSent();
    setPhone("");
    setContent("");
    setRows(1);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && e.shiftKey) {
      e.preventDefault();
      handleSubmit(e as any);
    } else if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      // Add newline to content and increase rows up to MAX_ROWS
      setContent((prev) => prev + "\n");
      setRows((prev) => Math.min(prev + 1, MAX_ROWS));
    }
  };

  const isOverLimit = content.length > 280;

  return (
    <div className={`w-full border-t ${theme.border}`}>
      <div className={`${darkMode ? "bg-[#141414]" : "bg-white"}`}>
        <div className="w-full lg:max-w-[60%] mx-auto px-4 py-4">
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
          <form onSubmit={handleSubmit} className="w-full">
            {/* Phone Input - On mobile, full width in separate row */}
            <div className="block lg:hidden mb-2">
              <input
                type="tel"
                placeholder="Phone (optional)"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                className={`w-full bg-transparent text-sm focus:outline-none border rounded-lg px-3 py-2 ${
                  darkMode
                    ? "bg-[#252525] border-zinc-700 text-white placeholder-zinc-500"
                    : "bg-zinc-100 border-zinc-300 text-black placeholder-zinc-400"
                }`}
              />
            </div>

            {/* Message Input Area */}
            <div
              className={`flex items-center gap-2 w-full ${
                darkMode ? "bg-[#252525]" : "bg-zinc-100"
              } rounded-lg px-4 py-2`}
            >
              {/* Phone Input - Desktop only, inline */}
              <input
                type="tel"
                placeholder="Phone"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                className={`hidden lg:block w-24 bg-transparent text-xs focus:outline-none shrink-0 ${
                  darkMode
                    ? "text-white placeholder-zinc-500"
                    : "text-black placeholder-zinc-400"
                }`}
              />
              <div
                className={`hidden lg:block w-px h-4 shrink-0 ${
                  darkMode ? "bg-zinc-700" : "bg-zinc-300"
                }`}
              />
              {/* Message Input - Larger Width */}
              <textarea
                ref={textareaRef}
                placeholder={
                  activeTab === "offered"
                    ? "Share what you're offering..."
                    : "Describe what you're looking for..."
                }
                value={content}
                onChange={(e) => setContent(e.target.value)}
                onKeyDown={handleKeyDown}
                maxLength={280}
                rows={rows}
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
                } transition-all hover:opacity-90 shrink-0 ${
                  timeLeft > 0 || !content.trim() || isOverLimit
                    ? "opacity-40"
                    : ""
                }`}
              >
                <Send className="w-4 h-4" />
              </button>
            </div>
            <div className={`text-[10px] ${theme.textMuted} mt-1 px-1`}>
              {rows > 1 ? "Enter to add line • " : ""}
              Shift+Enter to send • {content.length}/280 • Messages stay for 48
              hours
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};
