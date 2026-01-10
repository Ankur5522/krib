# Security Implementation Documentation

This document describes the comprehensive security system implemented for the Kirb application.

## Overview

The security system implements a multi-layered defense against spam, scraping, and scams without requiring user login. It uses Redis for rate limiting and ban management.

## Architecture

### Layer 1: Identification (Composite ID)

**Components:**

- `CompositeKeyGenerator` in [security/composite_key.rs](server/src/security/composite_key.rs)

**How it works:**

1. Frontend integrates ThumbmarkJS to generate a browser fingerprint
2. Backend extracts user IP address and fingerprint from each request
3. Generates a composite key: `SHA256(IP + Fingerprint + ServerSecret)`
4. This key uniquely identifies users across requests

**Why it's secure:**

- Even if IP changes, fingerprint remains consistent
- Server secret prevents key prediction
- Hash provides anonymity while maintaining identification

### Layer 2: Rate Limiting (Sliding Window)

**Components:**

- `RateLimiter` in [security/rate_limiter.rs](server/src/security/rate_limiter.rs)
- Redis sorted sets for sliding window implementation

**Rate Limits:**

- **Post Message**: 1 post per 60 seconds per user
- **Contact Reveal**: 5 reveals per hour per user
- **Burst Protection**: 20 requests per 2 seconds (global)

**How it works:**

1. Uses Redis sorted sets with timestamps as scores
2. Sliding window: removes old entries, counts current requests
3. If limit exceeded, request is blocked with retry-after time
4. Burst protection automatically blocks IPs for 30 minutes

### Layer 3: Shadowban System

**Components:**

- `ShadowbanManager` in [security/shadowban.rs](server/src/security/shadowban.rs)

**Features:**

- Shadowbanned users can post messages, but they're not broadcast
- Violation tracking system with auto-shadowban
- Configurable ban durations (temporary or permanent)
- Admin tracking with ban reasons

**Auto-Shadowban Logic:**

- After 3 content violations → 24-hour shadowban
- Violations reset after 24 hours if no new violations
- Honeypot triggers → permanent shadowban

### Layer 4: Content Filtering

**Components:**

- `ContentFilter` in [security/content_filter.rs](server/src/security/content_filter.rs)

**Filters:**

1. **Scam URLs**: Blocks Telegram bots, suspicious URL shorteners

   - `t.me`, `telegram.me`, `bit.ly`, `tinyurl.com`, etc.

2. **Embedded Phone Numbers**: Forces users to use dedicated phone field

   - Matches various phone number patterns

3. **Spam Phrases**: Detects common scam/spam language

   - "contact me on telegram", "dm me", "whatsapp only", etc.
   - "limited offer", "act fast", "make money fast", etc.

4. **Suspicious Patterns**:
   - Excessive character repetition (>5 consecutive chars)
   - Excessive capitalization (>70% caps)

### Layer 5: Honeypot Defense

**Implementation:**

- Hidden `website` field in POST request
- CSS hides it from humans, bots fill it
- If filled → permanent shadowban
- Located in: [models.rs](server/src/models.rs) (`PostMessageRequest.website`)

## Setup Instructions

### 1. Install Redis

**Option A: Docker (Recommended)**

```bash
docker run -d --name redis -p 6379:6379 redis:alpine
```

**Option B: Local Installation**

```bash
# Ubuntu/Debian
sudo apt-get install redis-server

# macOS
brew install redis
brew services start redis
```

### 2. Configure Environment Variables

Copy `.env.example` to `.env`:

```bash
cd server
cp .env.example .env
```

Edit `.env`:

```env
# Redis URL - use your Redis connection string
REDIS_URL=redis://127.0.0.1:6379

# Generate a strong secret with: openssl rand -hex 32
SERVER_SECRET=your-strong-random-secret-here
```

### 3. Frontend Integration

The frontend needs to send the browser fingerprint in the `X-Browser-Fingerprint` header:

```typescript
// Install ThumbmarkJS
npm install @thumbmarkjs/thumbmarkjs

// In your frontend code
import Thumbmark from '@thumbmarkjs/thumbmarkjs';

// Generate fingerprint once on app load
const fingerprint = await Thumbmark.get();

// Include in all API requests
fetch('http://localhost:3001/messages', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'X-Browser-Fingerprint': fingerprint,
  },
  body: JSON.stringify({
    browser_id: browserId,
    message: 'Hello',
    message_type: 'offered',
  }),
});
```

Add the honeypot field to your form (hidden with CSS):

```html
<input
  type="text"
  name="website"
  style="position: absolute; left: -9999px;"
  tabindex="-1"
  autocomplete="off"
/>
```

## API Error Responses

### Rate Limit Exceeded (429)

```json
{
  "error": "Rate limit exceeded",
  "retry_after": 1704892800
}
```

### Content Policy Violation (403)

```json
{
  "error": "Content policy violation",
  "reason": "Message contains suspicious URL"
}
```

### IP Blocked (429)

```
"IP address temporarily blocked due to excessive requests"
```

## Redis Key Structure

The system uses the following Redis key patterns:

- `ratelimit:post:<composite_key>` - Post rate limiting (sorted set)
- `ratelimit:reveal:<composite_key>` - Contact reveal rate limiting (sorted set)
- `ratelimit:burst:<composite_key>` - Burst protection (sorted set)
- `blocked:ip:<ip_address>` - Blocked IP addresses (string with TTL)
- `shadowban:<composite_key>` - Shadowbanned users (string with reason)
- `violations:<composite_key>` - Violation count (integer with 24h TTL)

## Monitoring & Administration

### Check if a user is shadowbanned

```redis
GET shadowban:<composite_key>
TTL shadowban:<composite_key>
```

### Check violation count

```redis
GET violations:<composite_key>
```

### Check IP block status

```redis
GET blocked:ip:192.168.1.1
TTL blocked:ip:192.168.1.1
```

### View rate limit usage

```redis
ZRANGE ratelimit:post:<composite_key> 0 -1 WITHSCORES
ZCOUNT ratelimit:reveal:<composite_key> -inf +inf
```

### Manual shadowban (via Redis CLI)

```redis
# Permanent ban
SET shadowban:<composite_key> "manual_ban_reason" EX 315360000

# 24-hour ban
SET shadowban:<composite_key> "spam_violation" EX 86400
```

### Clear a shadowban

```redis
DEL shadowban:<composite_key>
```

## Testing the Security System

### Test Rate Limiting

```bash
# Send multiple requests quickly
for i in {1..5}; do
  curl -X POST http://localhost:3001/messages \
    -H "Content-Type: application/json" \
    -H "X-Browser-Fingerprint: test123" \
    -d '{"browser_id":"test","message":"Test","message_type":"offered"}'
done
```

### Test Content Filtering

```bash
# Test scam URL detection
curl -X POST http://localhost:3001/messages \
  -H "Content-Type: application/json" \
  -H "X-Browser-Fingerprint: test123" \
  -d '{"browser_id":"test","message":"Contact me on t.me/scambot","message_type":"offered"}'
```

### Test Honeypot

```bash
# Fill honeypot field (should result in ban)
curl -X POST http://localhost:3001/messages \
  -H "Content-Type: application/json" \
  -H "X-Browser-Fingerprint: bot123" \
  -d '{"browser_id":"bot","message":"Test","message_type":"offered","website":"filled"}'
```

## Production Deployment Checklist

- [ ] Generate strong `SERVER_SECRET` using `openssl rand -hex 32`
- [ ] Use production Redis instance (Redis Cloud, AWS ElastiCache, etc.)
- [ ] Enable Redis authentication (update `REDIS_URL` with credentials)
- [ ] Enable Redis TLS/SSL for encrypted connections
- [ ] Set up Redis persistence (RDB or AOF)
- [ ] Configure Redis maxmemory policy (e.g., `allkeys-lru`)
- [ ] Monitor Redis memory usage and performance
- [ ] Set up logging and alerting for security events
- [ ] Review and adjust rate limits based on traffic patterns
- [ ] Keep `.env` file out of version control
- [ ] Use environment variables in production (not `.env` file)

## Security Considerations

1. **Server Secret**: Must be cryptographically random and kept secret
2. **Redis Security**: Use authentication and encryption in production
3. **IP Detection**: Use trusted proxy headers (X-Forwarded-For) with validation
4. **Rate Limits**: Adjust based on legitimate user behavior
5. **Shadowban Ethics**: Use responsibly, provide appeal mechanism
6. **GDPR Compliance**: Composite keys are hashed, but consider data retention

## Performance Impact

- **Redis Latency**: ~1-2ms per operation (local), ~5-10ms (cloud)
- **Memory Usage**: ~1-5KB per active user in Redis
- **Request Overhead**: ~5-15ms added to each request for security checks
- **Recommended**: Use Redis connection pooling (already implemented)

## Future Enhancements

- [ ] Machine learning-based scam detection
- [ ] CAPTCHA integration for suspicious users
- [ ] Admin dashboard for managing bans and viewing analytics
- [ ] WebSocket rate limiting
- [ ] GeoIP-based blocking
- [ ] Advanced fingerprinting (canvas, WebGL, audio)
- [ ] Reputation scoring system
