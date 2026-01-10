import { useEffect, useRef, useState } from "react";
import { useChatStore } from "../store/useChatStore";
import { MessageItem } from "./MessageItem";
import { Home, Search, Loader2 } from "lucide-react";
import { cn } from "../lib/utils";

const MESSAGES_PER_PAGE = 20;

export const MessageList = () => {
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

  if (filteredMessages.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400 animate-in fade-in">
        <div
          className={cn(
            "w-24 h-24 rounded-2xl flex items-center justify-center mb-6 shadow-lg",
            activeTab === "offered"
              ? "bg-linear-to-br from-blue-100 to-blue-200"
              : "bg-linear-to-br from-purple-100 to-pink-200"
          )}
        >
          {activeTab === "offered" ? (
            <Home size={48} className="text-blue-600" />
          ) : (
            <Search size={48} className="text-purple-600" />
          )}
        </div>
        <p className="text-2xl font-bold text-gray-700">No messages yet</p>
        <p className="text-base text-gray-500 mt-2">Be the first to post!</p>
      </div>
    );
  }

  return (
    <div
      ref={scrollContainerRef}
      onScroll={handleScroll}
      className="flex flex-col gap-3 items-end overflow-y-auto h-full px-2 py-2"
      style={{ maxHeight: "calc(100vh - 280px)" }}
    >
      {hasMore && (
        <div className="w-full flex justify-center py-2">
          {isLoadingMore ? (
            <Loader2 className="animate-spin text-gray-400" size={20} />
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
              className="text-sm text-gray-500 hover:text-gray-700 transition-colors"
            >
              Load more messages
            </button>
          )}
        </div>
      )}
      {displayedMessages.map((msg) => (
        <MessageItem key={msg.id} message={msg} />
      ))}
    </div>
  );
};
