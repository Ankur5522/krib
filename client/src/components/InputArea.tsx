import { useState, useEffect } from "react";
import { Send, Clock, AlertCircle } from "lucide-react";
import { useChatStore } from "../store/useChatStore";
import { type MessageType } from "../types";
import { cn } from "../lib/utils";

interface InputAreaProps {
  onSendMessage: (content: string, phone: string, type: MessageType) => void;
  error?: string | null;
}

const COOLDOWN_MS = 60 * 1000;

export const InputArea = ({ onSendMessage, error }: InputAreaProps) => {
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

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    // Allow Enter to create newlines, but don't submit the form
    if (e.key === "Enter" && !e.ctrlKey && !e.metaKey) {
      return; // Let the default behavior (newline) happen
    }
  };

  return (
    <div className="border-t border-gray-200 bg-white/80 backdrop-blur-sm px-4 py-3">
      <form onSubmit={handleSubmit} className="flex items-center gap-3">
        <input
          type="tel"
          placeholder="WhatsApp (optional)"
          value={phone}
          onChange={(e) => setPhone(e.target.value)}
          className="w-40 px-3 py-3 text-sm bg-white rounded-lg border border-gray-300 focus:border-blue-400 focus:ring-1 focus:ring-blue-200 transition-all outline-none h-11"
          style={{ fontSize: "16px" }}
        />

        <div className="flex-1 relative">
          <textarea
            className="w-full h-11 px-3 py-3 pr-16 bg-white rounded-lg resize-none focus:outline-none focus:ring-1 focus:ring-blue-200 focus:border-blue-400 transition-all text-sm border border-gray-300"
            placeholder={
              activeTab === "offered"
                ? "Describe your room offer..."
                : "What are you looking for..."
            }
            value={content}
            onChange={(e) => setContent(e.target.value)}
            onKeyDown={handleKeyDown}
            maxLength={280}
            style={{ fontSize: "16px" }}
          />
          <div
            className={cn(
              "absolute bottom-2 right-2 text-xs font-medium px-2 py-0.5 rounded",
              content.length > 250
                ? "bg-red-100 text-red-600"
                : "bg-gray-100 text-gray-500"
            )}
          >
            {content.length}/280
          </div>
        </div>

        <button
          type="submit"
          disabled={timeLeft > 0 || !content.trim() || isOverLimit}
          className={cn(
            "h-11 px-6 py-2 rounded-lg font-semibold text-white transition-all flex items-center gap-2 text-sm whitespace-nowrap",
            timeLeft > 0
              ? "bg-gray-400 cursor-not-allowed"
              : activeTab === "offered"
              ? "bg-linear-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 active:scale-95 shadow-md hover:shadow-lg"
              : "bg-linear-to-r from-purple-500 to-pink-600 hover:from-purple-600 hover:to-pink-700 active:scale-95 shadow-md hover:shadow-lg"
          )}
        >
          {timeLeft > 0 ? (
            <>
              <Clock size={18} className="animate-pulse" />
              <span>{timeLeft}s</span>
            </>
          ) : (
            <>
              <Send size={18} />
              <span>Post</span>
            </>
          )}
        </button>
      </form>
      {error && (
        <div className="mt-2 flex items-center gap-2 px-3 py-2 bg-red-50 border border-red-200 rounded-lg text-sm text-red-700">
          <AlertCircle size={16} className="flex-shrink-0" />
          <span>{error}</span>
        </div>
      )}
    </div>
  );
};
