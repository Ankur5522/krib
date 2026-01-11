## Axum 0.7 Backend Security Hardening - Implementation Summary

### ✅ Completed Security Enhancements

#### 1. **Content Sanitization (XSS Prevention)**

- **Dependency**: `ammonia 4.0` added to Cargo.toml
- **Location**: [server/src/models.rs](server/src/models.rs#L5-L10)
- **Implementation**:
  - `sanitize_html()` function uses Ammonia to strip dangerous HTML/JavaScript
  - Applied automatically in `ChatMessage::new()` during message creation
  - Safe HTML tags are preserved while XSS vectors are removed
  - Additional `sanitize_message()` method for post-load sanitization

```rust
// Automatically sanitized when creating messages
let message = ChatMessage::new(
    browser_id,
    "<script>alert('xss')</script>Hello", // Input
    message_type,
    phone,
    location,
);
// message.message will be cleaned
```

---

#### 2. **IP Rate Limiting (Governor)**

- **Dependency**: `governor 0.6` added to Cargo.toml
- **Location**: [server/src/security/governor_rate_limiter.rs](server/src/security/governor_rate_limiter.rs)
- **Implementation**:
  - 50 requests per minute per IP address
  - Integrated in [server/src/security/middleware.rs](server/src/security/middleware.rs#L115-L125)
  - Middleware `burst_protection_middleware` enforces the limit
  - Returns `429 Too Many Requests` when limit exceeded
  - Real IP extracted from X-Forwarded-For and Cf-Connecting-Ip headers

```rust
// The middleware checks governor rate limit on every request
if !state.governor_limiter.check_ip_rate_limit(&ctx.ip_address) {
    return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded: 50 requests per minute per IP")
}
```

---

#### 3. **Environment Variables Management**

- **Dependency**: `dotenvy 0.15` (replaced `dotenv`)
- **Location**: [server/src/main.rs](server/src/main.rs#L17-L25)
- **Configuration**: [server/.env](server/.env)
- **Required Variables**:
  - `SERVER_SECRET`: Strong random secret for composite key generation (required)
  - `REDIS_URL`: Connection string with Upstash credentials (required)
  - `ALLOWED_ORIGIN`: CORS origin domain (required in production)
  - `DATABASE_URL`: PostgreSQL connection (optional, not used yet)

```env
# server/.env
SERVER_SECRET=your-secure-random-secret-change-this-in-production
REDIS_URL=rediss://default:password@host:6379
ALLOWED_ORIGIN=https://yourdomain.com
```

**Setup Instructions**:

```bash
# Generate a secure SECRET
openssl rand -hex 32

# Copy .env file and update with production values
cp server/.env.example server/.env
# Edit with real credentials
```

---

#### 4. **CORS Security (Domain Restriction)**

- **Location**: [server/src/main.rs](server/src/main.rs#L50-L64)
- **Implementation**:
  - Uses `tower_http::cors::CorsLayer` with specific origin matching
  - Configured to accept only single domain from `ALLOWED_ORIGIN` env var
  - NO wildcard (\*) allowed in production
  - Allowed methods: GET, POST, OPTIONS
  - Allowed headers: Content-Type, Authorization
  - Max-age: 3600 seconds (1 hour)

```rust
let cors = CorsLayer::new()
    .allow_origin(
        allowed_origin.parse::<AllowOrigin>()
            .expect("Invalid ALLOWED_ORIGIN value")
    )
    .allow_methods([
        axum::http::Method::GET,
        axum::http::Method::POST,
        axum::http::Method::OPTIONS,
    ])
    .allow_headers([
        axum::http::header::CONTENT_TYPE,
        axum::http::header::AUTHORIZATION,
    ])
    .max_age(Duration::from_secs(3600));
```

---

#### 5. **Redis Connection (Upstash)**

- **Status**: Using Upstash managed Redis (no additional setup needed)
- **Security Features Already Configured**:
  - Secure TLS/SSL connection (rediss:// protocol)
  - Upstash handles password authentication automatically
  - Connection managed via [server/src/redis_client.rs](server/src/redis_client.rs)

---

### 📋 Configuration Checklist for Production

- [ ] Generate secure `SERVER_SECRET` with `openssl rand -hex 32`
- [ ] Set `ALLOWED_ORIGIN` to your production domain (e.g., `https://yourdomain.com`)
- [ ] Verify `REDIS_URL` uses Upstash credentials with `rediss://` protocol
- [ ] Deploy `.env` file securely (never commit to Git)
- [ ] Add `.env` to `.gitignore`:
  ```
  server/.env
  server/.env.local
  ```
- [ ] Test CORS by making requests from allowed domain
- [ ] Test rate limiting: curl multiple requests and verify 429 response after 50 requests/minute
- [ ] Verify sanitization by testing XSS payloads in message content

---

### 🧪 Testing the Hardening

#### Test XSS Sanitization:

```bash
curl -X POST http://localhost:3001/messages \
  -H "Content-Type: application/json" \
  -d '{
    "browser_id": "test123",
    "message": "<script>alert(\"xss\")</script>Hello",
    "message_type": "offered"
  }'
# The returned message will have sanitized content
```

#### Test IP Rate Limiting (50/minute):

```bash
# Run 51 requests in quick succession
for i in {1..51}; do
  curl http://localhost:3001/health
done
# Request 51 should get 429 Too Many Requests
```

#### Test CORS:

```bash
# From wrong domain (should fail)
curl -H "Origin: https://wrong-domain.com" \
  -H "Access-Control-Request-Method: POST" \
  http://localhost:3001/messages -v

# From correct domain (should succeed)
curl -H "Origin: http://localhost:5173" \
  -H "Access-Control-Request-Method: POST" \
  http://localhost:3001/messages -v
```

---

### 🔧 Updated Dependencies

```toml
ammonia = "4.0"           # HTML sanitization for XSS prevention
governor = "0.6"          # Rate limiting with token bucket algorithm
dotenvy = "0.15"          # Environment variable management
```

---

### 📚 Security Layers Active

1. **XSS Protection**: Ammonia sanitization on all message content
2. **Rate Limiting**:
   - Governor: 50 requests/minute per IP
   - Burst protection: 20 requests/2 seconds per fingerprint
   - Custom rate limiter: Per-endpoint limits in Redis
3. **IP Extraction**: Handles X-Forwarded-For, Cf-Connecting-Ip, and direct connections
4. **CORS**: Domain-specific, no wildcards in production
5. **Environment Security**: Required env vars prevent insecure defaults
6. **Shadowban System**: Persistent bans for repeat offenders
7. **Bot Detection**: Burst profiler detects suspicious patterns

---

### ⚠️ Production Deployment Notes

1. **Never commit `.env` to Git** - use environment variables in CI/CD
2. **Rotate `SERVER_SECRET`** periodically in production
3. **Monitor rate limit metrics** - adjust 50/minute if needed
4. **Test CORS thoroughly** before going live
5. **Use HTTPS everywhere** (rediss:// for Redis, https:// for CORS)
6. **Keep dependencies updated**: Run `cargo audit` regularly
7. **Log security events** to external service for compliance
