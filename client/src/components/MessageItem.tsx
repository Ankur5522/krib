import { type Message } from "../types";
import { formatDistanceToNow } from "date-fns";
import { ContactReveal } from "./ContactReveal";
import { Clock } from "lucide-react";
import { generateRandomName } from "../lib/randomNames";

interface MessageItemProps {
  message: Message;
  theme: any;
}

export const MessageItem = ({ message, theme }: MessageItemProps) => {
  const tryFormatDate = (timestamp: string) => {
    try {
      return formatDistanceToNow(new Date(timestamp), { addSuffix: true });
    } catch {
      return "Just now";
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
        <div>
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
      </div>

      {/* Message Content */}
      <p className="text-base leading-relaxed text-white mb-3 whitespace-pre-wrap break-words font-medium">
        {message.content}
      </p>

      {/* Card Footer */}
      <ContactReveal postId={message.id} theme={theme} />
    </article>
  );
};
