# Backend Integration Guide for ContactReveal Component

## Overview

The ContactReveal component enables lazy-loading of phone numbers. Here's what needs to be implemented on the Rust backend:

## 1. Message Storage

When a message is received with a phone number, it should be stored in the database with the following structure:

```rust
struct StoredMessage {
    id: String,              // Unique message ID (UUID)
    browser_id: String,      // Device ID
    message: String,         // Message content
    message_type: String,    // "offered" or "requested"
    phone: String,          // Phone number (encrypted recommended)
    timestamp: i64,         // Unix timestamp
}
```

## 2. New API Endpoint: GET /api/contact/{messageId}

This endpoint should:

- Accept a message ID as a path parameter
- Look up the message in the database
- Return the phone number associated with that message
- Response format:

```json
{
  "phone": "+1234567890"
}
```

### Implementation Example (Actix-web):

```rust
#[get("/api/contact/{message_id}")]
async fn get_contact(
    message_id: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    match db::get_message(&data.pool, &message_id).await {
        Ok(Some(message)) => HttpResponse::Ok().json(json!({
            "phone": message.phone
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "Message not found"
        })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch contact"
            }))
        }
    }
}
```

## 3. Message Broadcasting

When a message is posted:

1. Save it to the database with a unique ID
2. Include the message ID in the broadcasted message
3. Broadcast to the appropriate channel (offered/requested)

The frontend expects the message object to include the `id` field:

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "browser_id": "device-123",
  "message": "Nice 2BR apartment",
  "message_type": "offered",
  "timestamp": 1673052000,
  "phone": "optional_on_initial_broadcast"
}
```

## 4. Security Considerations

1. **Phone Number Visibility**: Consider whether the phone number should be visible on initial broadcast or only via the API endpoint
2. **Rate Limiting**: Add rate limiting to `/api/contact/{messageId}` to prevent abuse
3. **Phone Encryption**: Consider encrypting phone numbers in the database
4. **Access Control**: Ensure users can only request their own phone numbers (if desired)

## 5. Frontend Flow

```
User submits message + phone
    ↓
Backend stores in DB with ID
    ↓
Message broadcasted to channel with ID
    ↓
Frontend renders message with "Contact" button
    ↓
User clicks "Contact" button
    ↓
Frontend calls GET /api/contact/{messageId}
    ↓
Backend returns phone number
    ↓
Frontend caches it and shows phone + WhatsApp/Call buttons
```

## 6. CORS Headers

Ensure the Rust backend includes appropriate CORS headers:

```rust
.wrap(
    actix_cors::Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
)
```

## Questions for Implementation

1. Should phone numbers be visible in the initial broadcast, or only via the API?
2. Do you want to encrypt phone numbers in the database?
3. Should there be any rate limiting on the `/api/contact` endpoint?
4. Do you need any analytics on contact reveal requests?
