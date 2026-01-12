# Content Moderation Service

## Overview

The `ModerationService` is a comprehensive content moderation system for the Kirb rental platform. It provides multiple layers of content filtering to prevent spam, profanity, off-topic messages, and high-level policy violations.

## Features

### 1. **Profanity Filter**

- Detects English profanity patterns (damn, hell, crap, ass, bitch, etc.)
- Detects Hinglish (English transliteration of Hindi) offensive words
- Uses regex patterns for efficient detection
- Non-blocking and case-insensitive

**Example Hinglish patterns detected:**

- `bc`, `bhosdike`, `lodu`, `chutiya`, `gaandu`, `harami`
- `madarchod`, `behenchod`, `chakka`
- `randi`, `saali`, `ullu`

### 2. **Context/Relevance Check**

- Validates that messages are relevant to the rental platform
- Uses keyword-density analysis
- Requires at least 10% rental-related keywords for messages longer than 3 words

**Monitored keywords:**

- `room`, `flat`, `apartment`, `bhk`, `rent`, `rental`, `property`
- `location`, `available`, `looking`, `accommodation`, `deposit`
- `furnished`, `sharing`, `parking`, `tenant`, `landlord`

### 3. **OpenAI Moderation API Integration**

- Detects high-level policy violations (optional)
- Checks for:
  - Hate speech and hateful content
  - Harassment and bullying
  - Sexual content
  - Violence
- Requires `OPENAI_API_KEY` environment variable to be enabled

### 4. **Anti-Spam Protection**

- Limits external URLs to maximum 2 per message
- Blocks known scam domains:
  - `t.me`, `telegram.me` (Telegram links)
  - `bit.ly`, `tinyurl.com`, `goo.gl` (URL shorteners)
  - `rebrand.ly`, `ow.ly`, `lnk.co` (URL shorteners)
  - `clickbank.net` (Known scam platform)

## Integration

### In Handlers

The moderation service is automatically called in `post_message` handler before message is saved to Redis:

```rust
// Run comprehensive moderation checks
let moderation_result = state.moderation_service.moderate_message(&request.message).await;
if !moderation_result.is_allowed {
    // Increment violation count
    if let Ok(violations) = state.shadowban_manager
        .increment_violations(&security_ctx.composite_key)
        .await
    {
        // Auto-shadowban after 3 violations (24 hour ban)
        let _ = state.shadowban_manager
            .auto_shadowban_on_violations(&security_ctx.composite_key, 3, 86400)
            .await;
    }

    return Err((
        StatusCode::FORBIDDEN,
        Json(json!(ContentFilterError::new(
            moderation_result.reason.unwrap_or_else(|| "Content policy violation".to_string())
        )))
    ));
}
```

### In AppState

The service is initialized in [src/state.rs](../src/state.rs):

```rust
pub struct AppState {
    // ... other fields ...
    pub moderation_service: ModerationService,
}

impl AppState {
    pub async fn new(redis_url: &str, server_secret: String) -> Result<Self> {
        // ... other initialization ...

        // Initialize moderation service with optional OpenAI API key
        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        let moderation_service = ModerationService::new(openai_api_key);

        Ok(Self {
            // ... other fields ...
            moderation_service,
        })
    }
}
```

## Environment Configuration

### Required Environment Variables

None - the system works without any env vars.

### Optional Environment Variables

**`OPENAI_API_KEY`** - Enable OpenAI Moderation API checks

```bash
export OPENAI_API_KEY="sk-..."
```

When set, the system will call OpenAI's `/v1/moderations` endpoint for high-level policy violation detection.

## Error Handling

When moderation fails, the system:

1. **Returns HTTP 403 Forbidden** with error details
2. **Does NOT save** the message to Redis
3. **Does NOT broadcast** the message to other users
4. **Does NOT appear** on anyone's feed
5. **Increments violation count** for the user
6. **Auto-shadowbans** after 3 violations (24-hour ban)

### Response Format

```json
{
  "error": "Content policy violation",
  "reason": "Profanity or offensive language detected"
}
```

## Violation Types

- **Profanity** - Contains profane or vulgar words
- **OffTopic** - Message lacks rental-related context
- **Spam** - Multiple URLs or known scam domains
- **HateContent** - Hate speech (OpenAI API)
- **HarassmentContent** - Harassing language (OpenAI API)
- **SexualContent** - Sexual content (OpenAI API)
- **OpenAiViolation** - Generic OpenAI policy violation

## Testing

Unit tests are included in the moderation module:

```bash
cargo test security::moderation
```

### Test Coverage

- `test_is_relevant_to_rentals` - Keyword density checks
- `test_profanity_check` - Profanity detection
- `test_spam_multiple_urls` - Multiple URL detection
- `test_spam_scam_domains` - Scam domain blocking
- `test_valid_single_url` - Single URL allowance

## Performance Considerations

- **Regex compilation** happens once at startup via `Lazy` statics
- **OpenAI API calls** are asynchronous and optional
- **No blocking I/O** except for optional OpenAI calls
- **Typical check time** < 1ms (without OpenAI)
- **OpenAI check time** ~100-200ms (when enabled)

## Future Enhancements

1. **Configurable keyword lists** via Redis
2. **Machine Learning integration** for context understanding
3. **Multi-language support** beyond Hinglish
4. **User reputation scoring** based on violation history
5. **Webhook notifications** for moderation violations
6. **Custom regex patterns** per platform instance

## Security Notes

- All moderation violations are logged with user composite key
- Violations trigger automatic shadowbanning after threshold
- OpenAI API key is loaded from environment (never hardcoded)
- Rate limiting is applied separately via `RateLimiter`
- Shadowban manager prevents repeat violators from being visible

## Related Components

- [security/shadowban.rs](../src/security/shadowban.rs) - Automatic user banning
- [security/content_filter.rs](../src/security/content_filter.rs) - Basic content filtering
- [handlers.rs](../src/handlers.rs) - HTTP handlers that use moderation
- [state.rs](../src/state.rs) - AppState initialization
