import { useEffect, useRef, useState } from "react";
import { useChatStore } from "../store/useChatStore";
import { MessageItem } from "./MessageItem";
import { Home, Search, Loader2 } from "lucide-react";

interface MessageListProps {
  theme: any;
  darkMode: boolean;
  isLoading?: boolean;
}

const MESSAGES_PER_PAGE = 20;

export const MessageList = ({
  theme,
  darkMode,
  isLoading,
}: MessageListProps) => {
  const { messages, activeTab } = useChatStore();
  const [displayCount, setDisplayCount] = useState(MESSAGES_PER_PAGE);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const previousScrollHeight = useRef(0);
  const isAtBottom = useRef(true);

  const filteredMessages = messages.filter((m) => m.type === activeTab);
  const displayedMessages = filteredMessages.slice(-displayCount);
  const hasMore = filteredMessages.length > displayCount;

  // Auto-scroll to bottom on new messages if user is at bottom
  useEffect(() => {
    if (scrollContainerRef.current && isAtBottom.current) {
      scrollContainerRef.current.scrollTop =
        scrollContainerRef.current.scrollHeight;
    }
  }, [messages]);

  // Handle scroll to load more messages
  const handleScroll = () => {
    const container = scrollContainerRef.current;
    if (!container) return;

    // Check if user is at bottom (within 50px)
    const isUserAtBottom =
      container.scrollHeight - container.scrollTop - container.clientHeight <
      50;
    isAtBottom.current = isUserAtBottom;

    // Check if scrolled to top and there are more messages
    if (container.scrollTop === 0 && hasMore && !isLoadingMore) {
      setIsLoadingMore(true);
      previousScrollHeight.current = container.scrollHeight;

      // Simulate loading delay
      setTimeout(() => {
        setDisplayCount((prev) => prev + MESSAGES_PER_PAGE);
        setIsLoadingMore(false);
      }, 300);
    }
  };

  // Maintain scroll position after loading more messages
  useEffect(() => {
    if (scrollContainerRef.current && previousScrollHeight.current > 0) {
      const newScrollHeight = scrollContainerRef.current.scrollHeight;
      const scrollDiff = newScrollHeight - previousScrollHeight.current;
      scrollContainerRef.current.scrollTop = scrollDiff;
      previousScrollHeight.current = 0;
    }
  }, [displayCount]);

  // Show loading state
  if (isLoading) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center py-20">
        <div className="relative">
          <Loader2
            className={`w-12 h-12 ${theme.textSecondary} animate-spin`}
          />
        </div>
        <p className={`text-sm ${theme.textMuted} mt-4`}>Loading messages...</p>
      </div>
    );
  }

  if (filteredMessages.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-center py-20">
        <div
          className={`w-16 h-16 mx-auto mb-4 rounded-full ${theme.accentSoft} flex items-center justify-center`}
        >
          {activeTab === "offered" ? (
            <Home className="w-8 h-8" />
          ) : (
            <Search className="w-8 h-8" />
          )}
        </div>
        <h3 className="font-semibold mb-2">No posts yet</h3>
        <p className={`text-sm ${theme.textMuted}`}>Be the first to post!</p>
      </div>
    );
  }

  return (
    <div
      ref={scrollContainerRef}
      onScroll={handleScroll}
      className="flex flex-col gap-3 overflow-y-auto h-full px-2 py-2 custom-scrollbar"
      style={{
        maxHeight: "calc(100vh - 280px)",
        scrollbarWidth: "thin",
        scrollbarColor: darkMode
          ? "#3f3f46 transparent"
          : "#d4d4d8 transparent",
      }}
    >
      <style>{`
        .custom-scrollbar::-webkit-scrollbar {
          width: 6px;
        }
        .custom-scrollbar::-webkit-scrollbar-track {
          background: transparent;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb {
          background: ${darkMode ? "#3f3f46" : "#d4d4d8"};
          border-radius: 3px;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover {
          background: ${darkMode ? "#52525b" : "#a1a1aa"};
        }
      `}</style>
      {hasMore && (
        <div className="w-full flex justify-center py-2">
          {isLoadingMore ? (
            <Loader2 className={`animate-spin ${theme.textMuted}`} size={20} />
          ) : (
            <button
              onClick={() => {
                setIsLoadingMore(true);
                previousScrollHeight.current =
                  scrollContainerRef.current?.scrollHeight || 0;
                setTimeout(() => {
                  setDisplayCount((prev) => prev + MESSAGES_PER_PAGE);
                  setIsLoadingMore(false);
                }, 300);
              }}
              className={`text-sm ${theme.textSecondary} hover:${theme.text} transition-colors`}
            >
              Load more messages
            </button>
          )}
        </div>
      )}
      {displayedMessages.map((msg) => (
        <MessageItem key={msg.id} message={msg} theme={theme} />
      ))}
    </div>
  );
};
