import { getBrowserFingerprint } from "./fingerprint";

const API_BASE_URL =
  import.meta.env.VITE_BACKEND_URL || "http://localhost:5000";

export const WS_BASE_URL = (() => {
  const backendUrl = import.meta.env.VITE_BACKEND_URL || "ws://localhost:5000";
  console.log("🔍 VITE_BACKEND_URL:", import.meta.env.VITE_BACKEND_URL);

  // Convert https:// to wss://, http:// to ws://
  if (backendUrl.startsWith("https://")) {
    const wsUrl = backendUrl.replace("https://", "wss://");
    console.log("✅ WebSocket URL (HTTPS→WSS):", wsUrl);
    return wsUrl;
  }
  if (backendUrl.startsWith("http://")) {
    const wsUrl = backendUrl.replace("http://", "ws://");
    console.log("✅ WebSocket URL (HTTP→WS):", wsUrl);
    return wsUrl;
  }
  console.log("✅ WebSocket URL (default):", backendUrl);
  return backendUrl;
})();

/**
 * Get headers with browser fingerprint for API requests
 */
async function getHeaders(): Promise<HeadersInit> {
  const fingerprint = await getBrowserFingerprint();

  return {
    "Content-Type": "application/json",
    "X-Browser-Fingerprint": fingerprint,
  };
}

/**
 * POST request with fingerprint header
 */
export async function apiPost<T>(endpoint: string, body: any): Promise<T> {
  const headers = await getHeaders();

  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    method: "POST",
    headers,
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const errorData = await response.json().catch(() => ({}));

    // Include the full error data in the message for parsing
    const errorMessage =
      errorData.message ||
      errorData.error ||
      `HTTP ${response.status}: ${response.statusText}`;
    const fullError = JSON.stringify(errorData);

    throw new Error(`${errorMessage} ${fullError}`);
  }

  return response.json();
}

/**
 * GET request with fingerprint header
 */
export async function apiGet<T>(endpoint: string): Promise<T> {
  const fingerprint = await getBrowserFingerprint();

  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    method: "GET",
    headers: {
      "X-Browser-Fingerprint": fingerprint,
    },
  });

  if (!response.ok) {
    const errorData = await response.json().catch(() => ({}));

    // Include the full error data in the message for parsing
    const errorMessage =
      errorData.message ||
      errorData.error ||
      `HTTP ${response.status}: ${response.statusText}`;
    const fullError = JSON.stringify(errorData);

    throw new Error(`${errorMessage} ${fullError}`);
  }

  return response.json();
}

/**
 * Report a message
 */
export async function reportMessage(
  messageId: string,
  reportedBrowserId: string
): Promise<{ success: boolean; message: string; reports_on_ip: number }> {
  return apiPost("/api/report", {
    message_id: messageId,
    reported_browser_id: reportedBrowserId,
  });
}
