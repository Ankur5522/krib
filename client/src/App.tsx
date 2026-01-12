import { useEffect, useState } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { Moon, Sun, MessageCircle, MapPin, Search, X } from "lucide-react";
import { Header } from "./components/Header";
import { MessageList } from "./components/MessageList";
import { InputArea } from "./components/InputArea";
import PolicyDialog from "./components/PolicyDialog";
import { useChatStore } from "./store/useChatStore";
import { getDeviceId } from "./lib/utils";
import { apiGet, apiPost } from "./lib/api";
import { type Message, type MessageType } from "./types";
import stateAndCityData from "./data/stateandcity.json";

// Use localhost:3001
const WS_URL = import.meta.env.VITE_WS_URL || "ws://localhost:3001/ws";

interface BackendMessage {
  id: string;
  browser_id?: string;
  device_id?: string;
  message?: string;
  content?: string;
  message_type?: MessageType;
  type?: MessageType;
  timestamp: number | string;
  phone?: string;
}

function App() {
  const { addMessage, clearMessages, setCooldown } = useChatStore();
  const [postError, setPostError] = useState<string | null>(null);
  const [darkMode, setDarkMode] = useState(false);
  const [city, setCity] = useState<string>("");
  const [state, setState] = useState<string>("Detecting...");
  const [locationDenied, setLocationDenied] = useState(false);
  // const [isLoadingLocation, setIsLoadingLocation] = useState(true); // Removed unused variable
  const [showCitySearch, setShowCitySearch] = useState(false);
  const [citySearch, setCitySearch] = useState("");
  const [availableCities, setAvailableCities] = useState<string[]>([]);
  const [showStateSelection, setShowStateSelection] = useState(false);
  const [stateSearch, setStateSearch] = useState("");
  const [isLoadingMessages, setIsLoadingMessages] = useState(false);
  const [policyAccepted, setPolicyAccepted] = useState(() => {
    return localStorage.getItem("policyAccepted") === "true";
  });
  const [dailyStats, setDailyStats] = useState<{
    unique_ips: number;
    message_count: number;
  } | null>(null);
  // Handler for accepting policy
  const handleAcceptPolicy = () => {
    setPolicyAccepted(true);
    localStorage.setItem("policyAccepted", "true");
  };

  const { lastMessage, readyState } = useWebSocket(WS_URL, {
    shouldReconnect: () => true,
    reconnectAttempts: 10,
    reconnectInterval: 3000,
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

  // Check localStorage first, only request location if not found
  useEffect(() => {
    const savedCity = localStorage.getItem("krib_city");
    const savedState = localStorage.getItem("krib_state");

    if (savedCity && savedState) {
      // Load from localStorage
      console.log("Loading location from localStorage:", savedCity, savedState);
      setCity(savedCity);
      setState(savedState);
      // setIsLoadingLocation(false); // removed

      // Load available cities for the state
      const cities =
        (stateAndCityData as Record<string, string[]>)[savedState] || [];
      setAvailableCities(cities);
    } else {
      // No saved location, request it
      requestLocation();
    }

    // Fetch cooldown status on app load
    fetchCooldownStatus();
    // Fetch daily stats on app load
    fetchDailyStats();
    // Track visitor on app load
    trackVisitor();
  }, [setCooldown]);

  // Fetch daily stats periodically (every 10 seconds)
  useEffect(() => {
    const interval = setInterval(() => {
      fetchDailyStats();
    }, 10000);
    return () => clearInterval(interval);
  }, []);

  const fetchCooldownStatus = async () => {
    try {
      const data = await apiGet<{
        can_post: boolean;
        remaining_seconds: number;
      }>("/api/cooldown");
      if (!data.can_post && data.remaining_seconds > 0) {
        setCooldown(data.remaining_seconds);
      }
    } catch (e) {
      console.error("Failed to fetch cooldown status:", e);
    }
  };

  const fetchDailyStats = async () => {
    try {
      const data = await apiGet<{
        unique_ips: number;
        message_count: number;
      }>("/api/stats/daily");
      setDailyStats(data);
    } catch (e) {
      console.error("Failed to fetch daily stats:", e);
    }
  };

  const trackVisitor = async () => {
    try {
      // Check if we've already tracked this visitor today
      const today = new Date().toISOString().split("T")[0];
      const lastTrackedDate = localStorage.getItem("kirb_visitor_tracked_date");

      // Only track if we haven't tracked today yet
      if (lastTrackedDate !== today) {
        await apiPost("/api/track-visitor", {});
        localStorage.setItem("kirb_visitor_tracked_date", today);
        console.log("Visitor tracked for today");
      }
    } catch (e) {
      console.error("Failed to track visitor:", e);
    }
  };

  const requestLocation = () => {
    if ("geolocation" in navigator) {
      // setIsLoadingLocation(true); // removed
      setLocationDenied(false);

      navigator.geolocation.getCurrentPosition(
        async (position) => {
          const { latitude, longitude } = position.coords;
          try {
            // Using BigDataCloud free API - no key required
            const response = await fetch(
              `https://api.bigdatacloud.net/data/reverse-geocode-client?latitude=${latitude}&longitude=${longitude}&localityLanguage=en`
            );
            const data = await response.json();

            console.log("Location API response:", data);

            // Get state from the response
            const stateName = data.principalSubdivision || "Unknown";
            console.log("Detected state:", stateName);

            setState(stateName);

            // Get cities for the detected state from JSON
            const cities =
              (stateAndCityData as Record<string, string[]>)[stateName] || [];
            setAvailableCities(cities);

            // If state not found in our data, show denied
            if (cities.length === 0) {
              setLocationDenied(true);
              setState("State not in list");
            } else {
              setShowCitySearch(true);
            }

            // setIsLoadingLocation(false); // removed
          } catch (error) {
            console.error("Failed to get location:", error);
            setState("Unknown");
            setLocationDenied(true);
            // setIsLoadingLocation(false); // removed
          }
        },
        (error) => {
          console.error("Geolocation error:", error);
          setLocationDenied(true);
          // setIsLoadingLocation(false); // removed
          setState("Location Denied");
        },
        {
          enableHighAccuracy: false,
          timeout: 15000,
          maximumAge: 300000, // 5 minutes cache
        }
      );
    } else {
      setState("Not Supported");
      setLocationDenied(true);
      // setIsLoadingLocation(false); // removed
    }
  };

  // Fetch initial messages when app mounts and city is detected
  useEffect(() => {
    if (!city || locationDenied || showCitySearch) return;

    const fetchInitialMessages = async () => {
      setIsLoadingMessages(true);
      try {
        console.log("Fetching messages for city:", city);
        // Send location as query parameter to backend for filtering
        const data = await apiGet<Message[]>(
          `/messages?location=${encodeURIComponent(city)}`
        );

        // Clear existing messages first
        clearMessages();

        // Adapt messages from backend format (backend should already filter by location)
        if (Array.isArray(data)) {
          console.log(`Received ${data.length} messages for ${city}`);
          if (data.length > 0) {
            console.log(
              "Sample message structure:",
              JSON.stringify(data[0], null, 2)
            );
          }

          data.forEach((msg: BackendMessage) => {
            const adaptedMessage: Message = {
              id: msg.id,
              device_id: msg.browser_id || msg.device_id || "unknown",
              content: msg.message || msg.content || "",
              type: (msg.message_type || msg.type) as MessageType,
              timestamp:
                typeof msg.timestamp === "number"
                  ? new Date(msg.timestamp * 1000).toISOString()
                  : msg.timestamp,
              phone: msg.phone,
            };
            addMessage(adaptedMessage);
          });

          console.log(`Added ${data.length} messages for ${city}`);
        }
      } catch (e) {
        console.error("Failed to fetch initial messages", e);
      } finally {
        setIsLoadingMessages(false);
      }
    };

    fetchInitialMessages();
  }, [city, locationDenied, showCitySearch, addMessage, clearMessages]);

  // Handle incoming messages from WebSocket - only add if from same city
  useEffect(() => {
    if (lastMessage !== null) {
      try {
        const data = JSON.parse(lastMessage.data);
        // Adapter for Rust backend format to Frontend format
        // Rust sends: { id, browser_id, message, message_type, timestamp (number), location? }
        // Frontend expects: { id, device_id, content, type, timestamp (string), phone? }

        // Only add message if it's from the same city
        const messageLocation = data.location;
        if (messageLocation && messageLocation !== city) {
          console.log("Ignoring message from different city:", messageLocation);
          return;
        }

        // Don't add messages from the current user (already added optimistically)
        const currentDeviceId = getDeviceId();
        const messageBrowserId = data.browser_id || data.device_id;
        if (messageBrowserId === currentDeviceId) {
          console.log(
            "Ignoring own message from WebSocket (already added optimistically)"
          );
          return;
        }

        const adaptedMessage: Message = {
          id: data.id,
          device_id: data.browser_id || data.device_id || "unknown",
          content: data.message || data.content || "",
          type: (data.message_type || data.type) as MessageType,
          timestamp:
            typeof data.timestamp === "number"
              ? new Date(data.timestamp * 1000).toISOString()
              : data.timestamp,
          phone: data.phone,
        };

        addMessage(adaptedMessage);
      } catch (e) {
        console.error("Failed to parse message", e);
      }
    }
  }, [lastMessage, addMessage, city]);

  const handleSendMessage = async (
    content: string,
    phone: string,
    type: MessageType
  ) => {
    const deviceId = getDeviceId();
    setPostError(null);

    // Validate that city is set
    if (!city) {
      console.error("Cannot send message: city is not set");
      setPostError("Please select a city first");
      return;
    }

    // Create optimistic message
    const optimisticMessage: Message = {
      id: `temp-${Date.now()}`,
      device_id: deviceId,
      content: content,
      type: type,
      timestamp: new Date().toISOString(),
      phone: phone || undefined,
    };

    // Add message immediately to UI
    addMessage(optimisticMessage);

    // Rust expect: PostMessageRequest { browser_id, message, message_type, phone?, website?, location? }
    const payload = {
      browser_id: deviceId,
      message: content,
      message_type: type,
      phone: phone || undefined,
      location: city, // Send user's location
      website: "", // Honeypot field - leave empty for legitimate users
    };

    console.log(
      "Sending message with payload:",
      JSON.stringify(payload, null, 2)
    );
    console.log("Current city state:", city);
    console.log("City is empty?", !city);

    try {
      const response = await apiPost("/messages", payload);
      console.log("Message sent successfully. Response:", response);
    } catch (e) {
      console.error("Failed to send message:", e);

      // Try to parse the error message
      const errorMessage =
        e instanceof Error ? e.message : "Failed to send message";

      // Check if it's a rate limit error with JSON data
      try {
        // Extract JSON from error message if present
        const jsonMatch = errorMessage.match(/\{.*\}/);
        if (jsonMatch) {
          const errorData = JSON.parse(jsonMatch[0]);

          if (errorData.retry_after_seconds !== undefined) {
            // Update cooldown based on backend response
            setCooldown(errorData.retry_after_seconds);
            setPostError(
              errorData.message || "Please wait before posting again"
            );
            return;
          }
        }
      } catch {
        // If parsing fails, just show the original error
      }

      // For non-rate-limit errors, extract just the message part
      const displayMessage = errorMessage.split(" {")[0];
      setPostError(displayMessage);
    }
  };

  return (
    <div
      className={`min-h-screen ${theme.bg} ${theme.text} font-[-apple-system,BlinkMacSystemFont,'SF_Pro_Display','SF_Pro_Text',sans-serif] transition-colors duration-300`}
    >
      {/* Always visible header */}
      <header
        className={`fixed top-0 left-0 right-0 z-40 ${theme.bgSecondary} border-b ${theme.border} backdrop-blur-xl`}
      >
        <div className="w-full lg:max-w-[60%] mx-auto px-4 py-3">
          <div className="flex items-center justify-start">
            <div className="flex items-center gap-2">
              <div
                className={`w-8 h-8 rounded-xl ${theme.accent} flex items-center justify-center`}
              >
                <MessageCircle className="w-4 h-4" />
              </div>
              <span className="text-xl font-semibold tracking-tight">Krib</span>
            </div>
          </div>
        </div>
      </header>

      {!policyAccepted && (
        <PolicyDialog
          onAccept={handleAcceptPolicy}
          theme={theme}
          darkMode={darkMode}
        />
      )}
      {/* Location Selection Overlay */}
      {policyAccepted && (locationDenied || showCitySearch) && (
        <div
          className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4"
          onClick={(e) => {
            // Close when clicking on backdrop
            if (e.target === e.currentTarget && !locationDenied) {
              setShowCitySearch(false);
              setShowStateSelection(false);
            }
          }}
        >
          <div
            className={`${theme.bgCard} ${theme.border} border rounded-2xl p-8 max-w-md w-full ${theme.glow} relative`}
          >
            {/* Close Button - Only show when not denied (don't let users close if location is required) */}
            {!locationDenied && (
              <button
                onClick={() => {
                  setShowCitySearch(false);
                  setShowStateSelection(false);
                }}
                className={`absolute top-4 right-4 p-2 rounded-full ${theme.accentSoft} hover:opacity-80 transition-all`}
              >
                <X className="w-5 h-5" />
              </button>
            )}

            <MapPin className="w-16 h-16 mx-auto mb-4 text-blue-500" />
            <h2 className="text-2xl font-bold mb-3">Select Your City</h2>

            {locationDenied ? (
              <>
                <p className={`${theme.textSecondary} mb-6`}>
                  Location access is required. Click below to enable it.
                </p>
                <button
                  onClick={() => window.location.reload()}
                  className={`${theme.accent} w-full px-6 py-3 rounded-full font-semibold transition-all hover:opacity-90`}
                >
                  Allow Location
                </button>
                <p className={`${theme.textMuted} text-xs mt-4`}>
                  Make sure to click "Allow" when your browser asks for
                  permission
                </p>
              </>
            ) : showStateSelection ? (
              <>
                <p className={`${theme.textSecondary} mb-4 text-sm`}>
                  Select your state manually
                </p>

                <div className="relative mb-4">
                  <Search
                    className={`absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 ${theme.textMuted}`}
                  />
                  <input
                    type="text"
                    placeholder="Search state..."
                    value={stateSearch}
                    onChange={(e) => setStateSearch(e.target.value)}
                    className={`w-full pl-10 pr-4 py-3 rounded-lg border ${theme.border} ${theme.input} text-sm focus:outline-none focus:ring-2 focus:ring-blue-500`}
                  />
                </div>

                <div
                  className="max-h-64 overflow-y-auto space-y-1 custom-scrollbar"
                  style={{
                    scrollbarWidth: "thin",
                    scrollbarColor: darkMode
                      ? "#3f3f46 transparent"
                      : "#d4d4d8 transparent",
                  }}
                >
                  {Object.keys(stateAndCityData as Record<string, string[]>)
                    .filter((s) =>
                      s.toLowerCase().includes(stateSearch.toLowerCase())
                    )
                    .map((stateName) => (
                      <button
                        key={stateName}
                        onClick={() => {
                          setState(stateName);
                          const cities =
                            (stateAndCityData as Record<string, string[]>)[
                              stateName
                            ] || [];
                          setAvailableCities(cities);
                          setShowStateSelection(false);
                          setCitySearch("");
                          setStateSearch("");
                        }}
                        className={`w-full text-left px-4 py-3 rounded-lg ${theme.accentSoft} hover:opacity-80 transition-all text-sm`}
                      >
                        {stateName}
                      </button>
                    ))}
                </div>

                <button
                  onClick={() => setShowStateSelection(false)}
                  className={`w-full mt-4 px-4 py-2 rounded-lg border ${theme.border} ${theme.textMuted} hover:opacity-80 transition-all text-xs`}
                >
                  Back to City Selection
                </button>
              </>
            ) : (
              <>
                <div
                  className={`${theme.accentSoft} px-4 py-3 rounded-lg mb-4 flex items-center justify-between`}
                >
                  <div>
                    <p className={`text-xs ${theme.textMuted} mb-1`}>
                      Detected State
                    </p>
                    <p className="font-semibold">{state}</p>
                  </div>
                  <button
                    onClick={() => setShowStateSelection(true)}
                    className={`text-xs px-3 py-1.5 rounded-full ${theme.accent} hover:opacity-90 transition-all`}
                  >
                    Change
                  </button>
                </div>

                <div className="relative mb-4">
                  <Search
                    className={`absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 ${theme.textMuted}`}
                  />
                  <input
                    type="text"
                    placeholder="Search your city..."
                    value={citySearch}
                    onChange={(e) => setCitySearch(e.target.value)}
                    className={`w-full pl-10 pr-4 py-3 rounded-lg border ${theme.border} ${theme.input} text-sm focus:outline-none focus:ring-2 focus:ring-blue-500`}
                  />
                </div>

                <div
                  className="max-h-64 overflow-y-auto space-y-1 custom-scrollbar"
                  style={{
                    scrollbarWidth: "thin",
                    scrollbarColor: darkMode
                      ? "#3f3f46 transparent"
                      : "#d4d4d8 transparent",
                  }}
                >
                  {availableCities
                    .filter((c) =>
                      c.toLowerCase().includes(citySearch.toLowerCase())
                    )
                    .map((cityName) => (
                      <button
                        key={cityName}
                        onClick={() => {
                          setCity(cityName);
                          setShowCitySearch(false);
                          // Save to localStorage
                          localStorage.setItem("krib_city", cityName);
                          localStorage.setItem("krib_state", state);
                          console.log(
                            "Saved location to localStorage:",
                            cityName,
                            state
                          );
                        }}
                        className={`w-full text-left px-4 py-3 rounded-lg ${theme.accentSoft} hover:opacity-80 transition-all text-sm`}
                      >
                        {cityName}
                      </button>
                    ))}
                </div>

                {availableCities.filter((c) =>
                  c.toLowerCase().includes(citySearch.toLowerCase())
                ).length === 0 && (
                  <p className={`${theme.textMuted} text-center py-4 text-sm`}>
                    No cities found
                  </p>
                )}

                <button
                  onClick={() => {
                    localStorage.removeItem("krib_city");
                    localStorage.removeItem("krib_state");
                    window.location.reload();
                  }}
                  className={`w-full mt-4 px-4 py-2 rounded-lg border ${theme.border} ${theme.textMuted} hover:opacity-80 transition-all text-xs`}
                >
                  Detect Location Again
                </button>
              </>
            )}
          </div>
        </div>
      )}

      {/* Sticky Header */}
      {policyAccepted ? (
        <header
          className={`fixed top-0 left-0 right-0 z-50 ${theme.bgSecondary} border-b ${theme.border} backdrop-blur-xl`}
        >
          <div className="w-full lg:max-w-[60%] mx-auto px-4 py-3">
            {/* Top Row - Logo & Theme */}
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <div
                  className={`w-8 h-8 rounded-xl ${theme.accent} flex items-center justify-center`}
                >
                  <MessageCircle className="w-4 h-4" />
                </div>
                <span className="text-xl font-semibold tracking-tight">
                  Krib
                </span>
              </div>
              <div className="flex items-center gap-2">
                {/* City Display */}
                <button
                  onClick={() => setShowCitySearch(true)}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full ${theme.accentSoft} text-xs font-medium hover:opacity-80 transition-all`}
                  title="Click to change city"
                >
                  <MapPin className="w-3 h-3" />
                  <span>{city || "Select City"}</span>
                </button>

                {/* Daily Stats Display */}
                {dailyStats && (
                  <div
                    className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full ${theme.accentSoft} text-xs font-medium`}
                  >
                    <span title="Unique visitors today">
                      👥 {dailyStats.unique_ips}
                    </span>
                    <span>•</span>
                    <span title="Messages posted today">
                      💬 {dailyStats.message_count}
                    </span>
                  </div>
                )}

                {/* Connection Status */}
                {readyState !== ReadyState.OPEN && (
                  <div
                    className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full ${theme.accentSoft} text-xs font-medium`}
                  >
                    <div className="w-2 h-2 bg-yellow-400 rounded-full animate-pulse"></div>
                    <span>Connecting...</span>
                  </div>
                )}

                {/* Theme Toggle */}
                <button
                  onClick={() => setDarkMode(!darkMode)}
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

            {/* Tab Selector */}
            <Header theme={theme} />
          </div>
        </header>
      ) : null}

      {/* Message Stream */}
      {policyAccepted ? (
        <main className="pt-32 px-4 w-full lg:max-w-[60%] mx-auto pb-0">
          <MessageList
            theme={theme}
            darkMode={darkMode}
            isLoading={isLoadingMessages}
          />
        </main>
      ) : null}

      {/* Bottom Input Bar */}
      {policyAccepted ? (
        <InputArea
          onSendMessage={handleSendMessage}
          error={postError}
          theme={theme}
          darkMode={darkMode}
        />
      ) : null}
    </div>
  );
}

export default App;
