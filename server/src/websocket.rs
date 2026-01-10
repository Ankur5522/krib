use axum::extract::ws::{Message, WebSocket};
use crate::{models::ChatMessage, state::AppState};
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Get Redis pub/sub connection
    let client = state.redis.get_client();
    let conn_result = client.get_async_connection().await;
    
    let pubsub = match conn_result {
        Ok(conn) => conn.into_pubsub(),
        Err(e) => {
            eprintln!("Failed to create Redis connection for WebSocket: {}", e);
            return;
        }
    };

    // Subscribe to the Redis pub/sub channel
    let channel = state.get_pubsub_channel().to_string();
    
    // Task 1: Send messages to this client (Redis pub/sub receiver)
    let mut send_task = tokio::spawn(async move {
        let mut pubsub = pubsub;
        
        if let Err(e) = pubsub.subscribe(&channel).await {
            eprintln!("Failed to subscribe to Redis channel: {}", e);
            return;
        }
        
        let mut pubsub_stream = pubsub.on_message();
        
        while let Some(msg) = pubsub_stream.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to get payload from Redis message: {}", e);
                    continue;
                }
            };
            
            // Parse the message
            match serde_json::from_str::<ChatMessage>(&payload) {
                Ok(message) => {
                    // Strip phone number for privacy - only available via API
                    let broadcast_message = ChatMessage {
                        phone: None,
                        ..message
                    };
                    
                    match serde_json::to_string(&broadcast_message) {
                        Ok(json) => {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to serialize message: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse message from Redis: {}", e);
                }
            }
        }
    });
    
    // Task 2: Receive messages from this client (not implemented yet)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(_text) => {
                    // Future: Handle incoming messages from client
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });
    
    // Wait for either task to complete (which means the connection is closed)
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        },
        _ = &mut recv_task => {
            send_task.abort();
        },
    }
}
