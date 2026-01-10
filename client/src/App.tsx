import { useEffect, useState } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";
import { Header } from "./components/Header";
import { MessageList } from "./components/MessageList";
import { InputArea } from "./components/InputArea";
import { useChatStore } from "./store/useChatStore";
import { getDeviceId } from "./lib/utils";
import { apiGet, apiPost } from "./lib/api";
import { type Message, type MessageType } from "./types";

// Use localhost:3001
const WS_URL = "ws://localhost:3001/ws";

function App() {
  const { addMessage, clearMessages, setCooldown } = useChatStore();
  const [postError, setPostError] = useState<string | null>(null);

  const { sendMessage, lastMessage, readyState } = useWebSocket(WS_URL, {
    shouldReconnect: () => true,
    reconnectFiles: 10,
    reconnectInterval: 3000,
  });

  // Fetch initial messages when app mounts
  useEffect(() => {
    const fetchInitialMessages = async () => {
      try {
        const data = await apiGet<any[]>("/messages");

        // Clear existing messages and load initial ones
        clearMessages();

        // Adapt messages from backend format to frontend format
        if (Array.isArray(data)) {
          data.forEach((msg: any) => {
            const adaptedMessage: Message = {
              id: msg.id,
              device_id: msg.browser_id || msg.device_id,
              content: msg.message || msg.content,
              type: (msg.message_type || msg.type) as MessageType,
              timestamp:
                typeof msg.timestamp === "number"
                  ? new Date(msg.timestamp * 1000).toISOString()
                  : msg.timestamp,
              phone: msg.phone,
            };
            addMessage(adaptedMessage);
          });
        }
      } catch (e) {
        console.error("Failed to fetch initial messages", e);
      }
    };

    fetchInitialMessages();
  }, [addMessage, clearMessages]);

  // Handle incoming messages from WebSocket
  useEffect(() => {
    if (lastMessage !== null) {
      try {
        const data = JSON.parse(lastMessage.data);
        // Adapter for Rust backend format to Frontend format
        // Rust sends: { id, browser_id, message, message_type, timestamp (number) }
        // Frontend expects: { id, device_id, content, type, timestamp (string), phone? }

        const adaptedMessage: Message = {
          id: data.id,
          device_id: data.browser_id || data.device_id,
          content: data.message || data.content,
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
  }, [lastMessage, addMessage]);

  const handleSendMessage = async (
    content: string,
    phone: string,
    type: MessageType
  ) => {
    const deviceId = getDeviceId();
    setPostError(null);

    // Rust expect: PostMessageRequest { browser_id, message, message_type, phone?, website? }
    const payload = {
      browser_id: deviceId,
      message: content,
      message_type: type,
      phone: phone || undefined,
      website: "", // Honeypot field - leave empty for legitimate users
    };

    try {
      await apiPost("/messages", payload);
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
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-purple-900 font-sans">
      {/* Desktop Header with Branding */}
      <header className="bg-white/10 backdrop-blur-md border-b border-white/10 sticky top-0 z-50 shadow-xl">
        <div className="max-w-5xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center shadow-lg">
                <span className="text-white font-bold text-xl">K</span>
              </div>
              <div>
                <h1 className="text-2xl font-bold text-white tracking-tight">
                  Kirb
                </h1>
                <p className="text-xs text-blue-200">Find Your Perfect Room</p>
              </div>
            </div>
            {readyState !== ReadyState.OPEN && (
              <div className="flex items-center gap-2 px-4 py-2 bg-yellow-500/20 rounded-lg border border-yellow-500/30">
                <div className="w-2 h-2 bg-yellow-400 rounded-full animate-pulse"></div>
                <span className="text-sm text-yellow-200 font-medium">
                  Connecting...
                </span>
              </div>
            )}
          </div>
        </div>
      </header>

      {/* Main Content Area */}
      <div className="max-w-5xl mx-auto px-6 py-8">
        <div
          className="bg-white/95 backdrop-blur-sm rounded-2xl shadow-2xl flex flex-col"
          style={{ height: "calc(100vh - 12rem)" }}
        >
          <Header />
          <main className="flex-1 overflow-hidden">
            <MessageList />
          </main>
          <InputArea onSendMessage={handleSendMessage} error={postError} />
        </div>
      </div>
    </div>
  );
}

export default App;
