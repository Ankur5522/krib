import { useState, useEffect } from "react";
import {
  MapPin,
  Send,
  Home,
  Search,
  Moon,
  Sun,
  Clock,
  ChevronDown,
  MessageCircle,
  Flag,
} from "lucide-react";

function App() {
  const [darkMode, setDarkMode] = useState(true);
  const [activeTab, setActiveTab] = useState("offered");
  const [selectedCity, setSelectedCity] = useState("New York");
  const [showCityDropdown, setShowCityDropdown] = useState(false);
  const [newMessage, setNewMessage] = useState("");
  const [posts, setPosts] = useState({
    offered: [
      {
        id: 1,
        user: "Alex M.",
        time: "2m ago",
        type: "Room",
        price: "$1,200/mo",
        location: "Brooklyn",
        message:
          "Spacious room in 3BR apt. Rooftop access, near L train. Available Dec 1. DM for pics!",
        badge: "Available Now",
      },
      {
        id: 2,
        user: "Sarah K.",
        time: "8m ago",
        type: "Studio",
        price: "$1,850/mo",
        location: "Manhattan",
        message:
          "Bright studio in East Village. Exposed brick, pets OK. Flexible move-in.",
        badge: null,
      },
      {
        id: 3,
        user: "Mike R.",
        time: "15m ago",
        type: "1BR",
        price: "$2,400/mo",
        location: "Williamsburg",
        message:
          "Modern 1BR with balcony. W/D in unit. 6 or 12 month lease. No fee!",
        badge: "No Fee",
      },
      {
        id: 4,
        user: "Jenny L.",
        time: "32m ago",
        type: "Room",
        price: "$950/mo",
        location: "Queens",
        message:
          "Quiet room in family home. All utilities included. Professional preferred.",
        badge: null,
      },
      {
        id: 5,
        user: "David C.",
        time: "1h ago",
        type: "2BR",
        price: "$3,200/mo",
        location: "Chelsea",
        message:
          "Stunning 2BR with city views. Doorman building, gym included. Jan 1 move-in.",
        badge: "Premium",
      },
    ],
    requested: [
      {
        id: 1,
        user: "Chris P.",
        time: "5m ago",
        budget: "Up to $1,500",
        area: "Brooklyn/Queens",
        message:
          "Couple looking for 1BR or large room. Have a small dog. Need by Jan 15.",
        urgent: true,
      },
      {
        id: 2,
        user: "Emma W.",
        time: "12m ago",
        budget: "Up to $1,200",
        area: "Any borough",
        message:
          "Grad student seeking room in shared apt. Clean, quiet, 420 friendly preferred.",
        urgent: false,
      },
      {
        id: 3,
        user: "Tom H.",
        time: "28m ago",
        budget: "Up to $2,000",
        area: "Manhattan",
        message:
          "Remote worker relocating from Austin. Need furnished studio ASAP. 3-month min.",
        urgent: true,
      },
      {
        id: 4,
        user: "Lisa N.",
        time: "45m ago",
        budget: "Up to $900",
        area: "Bronx/Harlem",
        message:
          "Single mom with toddler. Looking for safe, quiet area. Can provide references.",
        urgent: false,
      },
    ],
  });

  const theme = darkMode
    ? {
        bg: "bg-[#0A0A0A]",
        bgSecondary: "bg-[#141414]",
        bgTertiary: "bg-[#1C1C1E]",
        bgCard: "bg-[#1C1C1E]",
        text: "text-white",
        textSecondary: "text-zinc-400",
        textMuted: "text-zinc-500",
        border: "border-zinc-800",
        segmentBg: "bg-[#1C1C1E]",
        segmentActive: "bg-[#2C2C2E]",
        accent: "bg-white text-black",
        accentSoft: "bg-zinc-800",
        input: "bg-[#1C1C1E] border-zinc-800 text-white placeholder-zinc-500",
        glow: "shadow-[0_0_30px_rgba(255,255,255,0.03)]",
      }
    : {
        bg: "bg-[#F5F5F7]",
        bgSecondary: "bg-white",
        bgTertiary: "bg-zinc-100",
        bgCard: "bg-white",
        text: "text-black",
        textSecondary: "text-zinc-600",
        textMuted: "text-zinc-400",
        border: "border-zinc-200",
        segmentBg: "bg-zinc-200/60",
        segmentActive: "bg-white",
        accent: "bg-black text-white",
        accentSoft: "bg-zinc-100",
        input: "bg-zinc-100 border-zinc-200 text-black placeholder-zinc-400",
        glow: "shadow-lg",
      };

  const cities = [
    "New York",
    "Los Angeles",
    "Chicago",
    "San Francisco",
    "Miami",
    "Austin",
    "Seattle",
    "Boston",
  ];

  const currentPosts = posts[activeTab];

  const handleToggleDarkMode = () => {
    setDarkMode((prev) => !prev);
  };

  const handleTabChange = (tab) => {
    setActiveTab(tab);
  };

  const handleCitySelect = (city) => {
    setSelectedCity(city);
    setShowCityDropdown(false);
  };

  const handleToggleCityDropdown = () => {
    setShowCityDropdown((prev) => !prev);
  };

  const handleMessageChange = (e) => {
    setNewMessage(e.target.value);
  };

  const handleSendPost = () => {
    if (!newMessage.trim()) return;
    const newPost = {
      id: Date.now(),
      user: "You",
      time: "Just now",
      message: newMessage,
      ...(activeTab === "offered"
        ? {
            type: "Room",
            price: "DM for price",
            location: selectedCity,
            badge: null,
          }
        : { budget: "Flexible", area: selectedCity, urgent: false }),
    };
    setPosts((prev) => ({
      ...prev,
      [activeTab]: [newPost, ...prev[activeTab]],
    }));
    setNewMessage("");
  };

  const handleCloseCityDropdown = () => {
    setShowCityDropdown(false);
  };

  return (
    <div
      className={`min-h-screen ${theme.bg} ${theme.text} font-[-apple-system,BlinkMacSystemFont,'SF_Pro_Display','SF_Pro_Text',sans-serif] transition-colors duration-300`}
    >
      {/* Sticky Header */}
      <header
        className={`fixed top-0 left-0 right-0 z-50 ${theme.bgSecondary} border-b ${theme.border} backdrop-blur-xl`}
      >
        <div className="max-w-lg mx-auto px-4 py-3">
          {/* Top Row - Logo & Theme */}
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <div
                className={`w-8 h-8 rounded-xl ${theme.accent} flex items-center justify-center`}
              >
                <MessageCircle className="w-4 h-4" />
              </div>
              <span className="text-xl font-semibold tracking-tight">
                RoomShout
              </span>
            </div>
            <div className="flex items-center gap-2">
              {/* City Selector */}
              <div className="relative">
                <button
                  onClick={handleToggleCityDropdown}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full ${theme.accentSoft} text-sm font-medium transition-all`}
                >
                  <MapPin className="w-3.5 h-3.5" />
                  <span>{selectedCity}</span>
                  <ChevronDown
                    className={`w-3.5 h-3.5 transition-transform ${
                      showCityDropdown ? "rotate-180" : ""
                    }`}
                  />
                </button>

                {showCityDropdown && (
                  <>
                    <div
                      className="fixed inset-0 z-40"
                      onClick={handleCloseCityDropdown}
                    />
                    <div
                      className={`absolute right-0 top-full mt-2 w-48 ${theme.bgCard} border ${theme.border} rounded-2xl overflow-hidden z-50 ${theme.glow}`}
                    >
                      {cities.map((city) => (
                        <button
                          key={city}
                          onClick={() => handleCitySelect(city)}
                          className={`w-full px-4 py-2.5 text-left text-sm hover:${
                            theme.bgTertiary
                          } transition-colors ${
                            selectedCity === city ? "font-semibold" : ""
                          }`}
                        >
                          {city}
                        </button>
                      ))}
                    </div>
                  </>
                )}
              </div>

              {/* Theme Toggle */}
              <button
                onClick={handleToggleDarkMode}
                className={`p-2 rounded-full ${theme.accentSoft} transition-all`}
              >
                {darkMode ? (
                  <Sun className="w-4 h-4" />
                ) : (
                  <Moon className="w-4 h-4" />
                )}
              </button>
            </div>
          </div>

          {/* Segmented Control */}
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
                onClick={() => handleTabChange("offered")}
                className={`flex-1 flex items-center justify-center gap-2 py-2.5 rounded-lg text-sm font-semibold transition-colors z-10 ${
                  activeTab === "offered" ? theme.text : theme.textSecondary
                }`}
              >
                <Home className="w-4 h-4" />
                Offered
              </button>
              <button
                onClick={() => handleTabChange("requested")}
                className={`flex-1 flex items-center justify-center gap-2 py-2.5 rounded-lg text-sm font-semibold transition-colors z-10 ${
                  activeTab === "requested" ? theme.text : theme.textSecondary
                }`}
              >
                <Search className="w-4 h-4" />
                Requested
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Message Stream */}
      <main className="pt-32 pb-24 px-4 max-w-lg mx-auto">
        <div className="space-y-3">
          {currentPosts.map((post) => (
            <article
              key={post.id}
              className={`${theme.bgCard} border ${theme.border} rounded-2xl p-4 transition-all hover:scale-[1.01] ${theme.glow}`}
            >
              {/* Card Header */}
              <div className="flex items-center gap-3 mb-3">
                <div
                  className={`w-10 h-10 rounded-full ${theme.accentSoft} flex items-center justify-center font-semibold text-sm`}
                >
                  {post.user.split(" ")[0][0]}
                  {post.user.split(" ")[1]?.[0] || ""}
                </div>
                <div>
                  <div className="font-semibold text-sm">{post.user}</div>
                  <div
                    className={`flex items-center gap-1 text-xs ${theme.textMuted}`}
                  >
                    <Clock className="w-3 h-3" />
                    {post.time}
                  </div>
                </div>
              </div>

              {/* Message Content */}
              <p
                className={`text-sm leading-relaxed ${theme.textSecondary} mb-3`}
              >
                {post.message}
              </p>

              {/* Card Footer */}
              <div className="flex items-center justify-end gap-2">
                <button
                  className={`flex items-center gap-1.5 px-3 py-2 rounded-full ${theme.accentSoft} text-xs font-medium transition-all hover:opacity-80`}
                >
                  <Flag className="w-3.5 h-3.5" />
                  Report
                </button>
                <button
                  className={`flex items-center gap-1.5 px-4 py-2 rounded-full ${theme.accent} text-xs font-semibold transition-all hover:opacity-90`}
                >
                  <MessageCircle className="w-3.5 h-3.5" />
                  Contact
                </button>
              </div>
            </article>
          ))}
        </div>

        {/* Empty State Placeholder */}
        {currentPosts.length === 0 && (
          <div className="text-center py-20">
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
            <p className={`text-sm ${theme.textMuted}`}>
              Be the first to post in {selectedCity}!
            </p>
          </div>
        )}
      </main>

      {/* Bottom Input Bar */}
      <div className={`fixed bottom-0 left-0 right-0 z-30`}>
        <div
          className={`${darkMode ? "bg-[#141414]" : "bg-white"} border-t ${
            theme.border
          }`}
        >
          <div className="max-w-lg mx-auto px-4 py-4">
            <div
              className={`flex items-center gap-3 ${
                darkMode ? "bg-[#252525]" : "bg-zinc-100"
              } rounded-full px-4 py-2`}
            >
              <input
                type="text"
                placeholder={
                  activeTab === "offered"
                    ? "Share what you're offering..."
                    : "Describe what you're looking for..."
                }
                value={newMessage}
                onChange={handleMessageChange}
                onKeyDown={(e) => e.key === "Enter" && handleSendPost()}
                className={`flex-1 bg-transparent text-sm focus:outline-none ${
                  darkMode
                    ? "text-white placeholder-zinc-500"
                    : "text-black placeholder-zinc-400"
                }`}
              />
              <button
                onClick={handleSendPost}
                className={`p-2 rounded-full ${
                  theme.accent
                } transition-all hover:opacity-90 ${
                  !newMessage.trim() ? "opacity-40" : ""
                }`}
              >
                <Send className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
