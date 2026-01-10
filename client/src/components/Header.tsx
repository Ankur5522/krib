import { Home, Search } from "lucide-react";
import { useChatStore } from "../store/useChatStore";
import { cn } from "../lib/utils";

export const Header = () => {
  const { activeTab, setTab } = useChatStore();

  return (
    <div className="border-b border-gray-200 bg-gradient-to-r from-gray-50 to-white">
      <div className="flex gap-2 p-3">
        <button
          onClick={() => setTab("offered")}
          className={cn(
            "flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-lg font-semibold text-sm transition-all duration-300",
            activeTab === "offered"
              ? "bg-blue-500 text-white shadow-md"
              : "bg-gray-100 text-gray-600 hover:bg-gray-200"
          )}
        >
          <Home size={18} />
          <span className="hidden sm:inline">Rooms Offered</span>
        </button>
        <button
          onClick={() => setTab("requested")}
          className={cn(
            "flex-1 flex items-center justify-center gap-2 py-2.5 px-4 rounded-lg font-semibold text-sm transition-all duration-300",
            activeTab === "requested"
              ? "bg-purple-500 text-white shadow-md"
              : "bg-gray-100 text-gray-600 hover:bg-gray-200"
          )}
        >
          <Search size={18} />
          <span className="hidden sm:inline">Rooms Requested</span>
        </button>
      </div>
    </div>
  );
};
