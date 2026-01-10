export type MessageType = "offered" | "requested";

export interface Message {
  id: string;
  type: MessageType;
  content: string;
  phone?: string;
  timestamp: string;
  device_id: string;
}

// Payload for sending to server
export interface MessagePayload {
  device_id: string;
  message: string; // Mapping 'content' to 'message' for backend compatibility if needed, but requirements say 'content'. I will use 'content' for internal, but maybe mapping at socket layer.
  // Wait, requirements say JSON Contract: { "type": ..., "content": ... }
  // I'll stick to requirement keys.
  type: MessageType;
  phone?: string;
  content: string;
}
