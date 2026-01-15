import { useEffect, useRef, useState } from "react";
import { useChatStore } from "../store/useChatStore";
import { type Message } from "../types";

import { WS_BASE_URL } from "../lib/api";

const getWsUrl = () => {
  const wsUrl = WS_BASE_URL || "ws://localhost:3001";
  return `${wsUrl}/ws`;
};
const WS_URL = getWsUrl();
const RECONNECT_DELAY = 3000;
const MAX_RECONNECT_ATTEMPTS = 10;

export const useWebSocket = () => {
  const { addMessage } = useChatStore();
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<number | undefined>(undefined);

  const connect = () => {
    try {
      const ws = new WebSocket(WS_URL);

      ws.onopen = () => {
        setIsConnected(true);
        setError(null);
        reconnectAttemptsRef.current = 0;
      };

      ws.onmessage = (event) => {
        try {
          const message: Message = JSON.parse(event.data);
          addMessage(message);
        } catch (err) {
          // Silently handle error
        }
      };

      ws.onerror = (event) => {
        console.error("WebSocket error:", event);
        setError("Connection error");
      };

      ws.onclose = () => {
        setIsConnected(false);
        wsRef.current = null;

        // Attempt to reconnect
        if (reconnectAttemptsRef.current < MAX_RECONNECT_ATTEMPTS) {
          reconnectAttemptsRef.current++;
          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, RECONNECT_DELAY);
        } else {
          setError("Failed to connect after multiple attempts");
        }
      };

      wsRef.current = ws;
    } catch (err) {
      setError("Failed to establish connection");
    }
  };

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return { isConnected, error };
};
