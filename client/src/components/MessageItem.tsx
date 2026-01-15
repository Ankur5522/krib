import { type Message } from "../types";
import { formatDistanceToNow } from "date-fns";
import { ContactReveal } from "./ContactReveal";
import { Clock, Flag } from "lucide-react";
import { generateRandomName } from "../lib/randomNames";
import { reportMessage } from "../lib/api";
import { useState, useEffect } from "react";
import { getBrowserFingerprint } from "../lib/fingerprint";
import type { Theme } from "./MessageList";

interface MessageItemProps {
  message: Message;
  theme: Theme;
}

// Helper functions for localStorage
const getReportedMessages = (): Set<string> => {
  try {
    const stored = localStorage.getItem("reported_messages");
    return stored ? new Set(JSON.parse(stored)) : new Set();
  } catch {
    return new Set();
  }
};

const addReportedMessage = (messageId: string) => {
  const reported = getReportedMessages();
  reported.add(messageId);
  localStorage.setItem("reported_messages", JSON.stringify([...reported]));
};

const hasReportedMessage = (messageId: string): boolean => {
  return getReportedMessages().has(messageId);
};

export const MessageItem = ({ message, theme }: MessageItemProps) => {
  const [isReporting, setIsReporting] = useState(false);
  const [isReported, setIsReported] = useState(false);

  // Check if this message was already reported on component mount
  useEffect(() => {
    const alreadyReported = hasReportedMessage(message.id);
    if (alreadyReported) {
      setIsReported(true);
    }
  }, [message.id]);

  const tryFormatDate = (timestamp: string) => {
    try {
      return formatDistanceToNow(new Date(timestamp), { addSuffix: true });
    } catch {
      return "Just now";
    }
  };

  const handleReport = async () => {
    if (isReporting || isReported) return;

    setIsReporting(true);

    try {
      // Check if this is our own message
      const fingerprint = await getBrowserFingerprint();

      if (fingerprint === message.device_id) {
        setIsReporting(false);
        return;
      }

      // Check if already reported this message
      if (hasReportedMessage(message.id)) {
        setIsReporting(false);
        setIsReported(true);
        return;
      }

      await reportMessage(message.id, message.device_id);

      // Mark as reported in localStorage
      addReportedMessage(message.id);
      setIsReported(true);
    } catch (error: unknown) {
      // Silently handle error
    } finally {
      setIsReporting(false);
    }
  };

  return (
    <article
      className={`${theme.bgCard} border ${theme.border} rounded-2xl p-4 transition-all hover:scale-[1.01] ${theme.glow}`}
    >
      {/* Card Header */}
      <div className="flex items-center gap-2 mb-3 opacity-60">
        <div
          className={`w-7 h-7 rounded-full ${theme.accentSoft} flex items-center justify-center font-medium text-[10px]`}
        >
          {message.device_id.substring(0, 2).toUpperCase()}
        </div>
        <div className="flex-1">
          <div className="font-medium text-[11px]">
            {generateRandomName(message.device_id)}
          </div>
          <div
            className={`flex items-center gap-1 text-[10px] ${theme.textMuted}`}
          >
            <Clock className="w-2.5 h-2.5" />
            {tryFormatDate(message.timestamp)}
          </div>
        </div>

        {/* Report Button */}
        <button
          type="button"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            handleReport();
          }}
          disabled={isReporting || isReported}
          className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg transition-all text-xs font-medium ${
            isReported
              ? "bg-red-500/40 text-red-400 cursor-not-allowed"
              : "bg-red-500/30 text-red-400 hover:bg-red-500/40 hover:text-red-300 cursor-pointer"
          } ${isReporting ? "opacity-50 cursor-wait" : "cursor-pointer"}`}
          title={isReported ? "Already Reported" : "Report message"}
        >
          <Flag className="w-3 h-3" style={{ pointerEvents: "none" }} />
          <span style={{ pointerEvents: "none" }}>
            {isReported ? "Reported" : "Report"}
          </span>
        </button>
      </div>

      {/* Message Content */}
      <p
        className={`text-base leading-relaxed ${theme.text} mb-3 whitespace-pre-wrap wrap-break-word font-medium`}
        dangerouslySetInnerHTML={{ __html: message.content }}
      />

      {/* Card Footer */}
      <ContactReveal
        postId={message.id}
        theme={theme}
        messageContent={message.content}
      />
    </article>
  );
};
