import { type Message } from "../types";
import { formatDistanceToNow } from "date-fns";
import { cn } from "../lib/utils";
import { ContactReveal } from "./ContactReveal";

interface MessageItemProps {
  message: Message;
}

export const MessageItem = ({ message }: MessageItemProps) => {
  const isOffered = message.type === "offered";

  const tryFormatDate = (timestamp: string) => {
    try {
      return formatDistanceToNow(new Date(timestamp), { addSuffix: true });
    } catch {
      return "Just now";
    }
  };

  return (
    <div
      className={cn(
        "rounded-2xl shadow-lg hover:shadow-xl transition-all duration-300 animate-in fade-in max-w-sm w-full",
        isOffered ? "bg-blue-500 text-white" : "bg-purple-500 text-white"
      )}
    >
      <div className="p-4">
        <div className="flex items-baseline justify-between mb-2">
          <span className="text-xs font-semibold opacity-80">
            {tryFormatDate(message.timestamp)}
          </span>
        </div>

        <p className="text-sm leading-snug mb-3 whitespace-pre-wrap break-words">
          {message.content}
        </p>

        <ContactReveal postId={message.id} />
      </div>
    </div>
  );
};
