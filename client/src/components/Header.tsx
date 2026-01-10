import { Home, Search } from "lucide-react";
import { useChatStore } from "../store/useChatStore";

interface HeaderProps {
  theme: any;
}

export const Header = ({ theme }: HeaderProps) => {
  const { activeTab, setTab } = useChatStore();

  return (
    <div className={`relative ${theme.segmentBg} rounded-xl p-1`}>
      {/* Sliding Background */}
      <div
        className={`absolute top-1 bottom-1 w-[calc(50%-4px)] ${theme.segmentActive} rounded-lg transition-all duration-300 ease-out ${theme.glow}`}
        style={{
          left: activeTab === "offered" ? "4px" : "calc(50% + 0px)",
        }}
      />

      <div className="relative flex">
        <button
          onClick={() => setTab("offered")}
          className={`flex-1 flex items-center justify-center gap-2 py-2.5 rounded-lg text-sm font-semibold transition-colors z-10 ${
            activeTab === "offered" ? theme.text : theme.textSecondary
          }`}
        >
          <Home className="w-4 h-4" />
          Offered
        </button>
        <button
          onClick={() => setTab("requested")}
          className={`flex-1 flex items-center justify-center gap-2 py-2.5 rounded-lg text-sm font-semibold transition-colors z-10 ${
            activeTab === "requested" ? theme.text : theme.textSecondary
          }`}
        >
          <Search className="w-4 h-4" />
          Requested
        </button>
      </div>
    </div>
  );
};
