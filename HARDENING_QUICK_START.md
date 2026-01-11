## Security Hardening Quick Reference

### 🔐 Implemented Protections

| Feature            | Implementation           | Config                                                  |
| ------------------ | ------------------------ | ------------------------------------------------------- |
| **XSS Prevention** | Ammonia HTML sanitizer   | Auto-applied to messages                                |
| **Rate Limiting**  | Governor (50/min per IP) | Extractsreal IP from X-Forwarded-For                    |
| **CORS**           | Domain-specific          | ALLOWED_ORIGIN env var                                  |
| **Environment**    | Dotenvy (.env files)     | Required vars: SERVER_SECRET, REDIS_URL, ALLOWED_ORIGIN |

---

### 📝 Setup Instructions

#### 1. Install Dependencies

```bash
cd /home/ankur/Documents/kirb/server
cargo build
```

#### 2. Configure .env File

```bash
# Generate secure secret
openssl rand -hex 32

# Edit server/.env with:
SERVER_SECRET=<your-generated-secret>
REDIS_URL=rediss://default:<password>@host:6379
ALLOWED_ORIGIN=https://yourdomain.com
```

#### 3. Deploy

```bash
# Run server
cargo run

# Or build release
cargo build --release
./target/release/kirb-server
```

---

### ✅ Verification Checklist

- [ ] `.env` file created with real values
- [ ] `SERVER_SECRET` is a strong 64-character hex string
- [ ] `ALLOWED_ORIGIN` matches your production domain
- [ ] `REDIS_URL` uses `rediss://` protocol (Upstash)
- [ ] Server starts without warnings about missing env vars
- [ ] CORS works from your frontend domain
- [ ] Rate limiting kicks in after 50 requests/minute
- [ ] Message content is sanitized (test with `<script>` tags)

---

### 🔍 Testing Commands

**Check compilation:**

```bash
cd server && cargo check
```

**Run server:**

```bash
cd server && cargo run
```

**Test rate limiting (50/minute per IP):**

```bash
for i in {1..60}; do curl http://localhost:3001/health; done
```

**Test CORS:**

```bash
curl -H "Origin: https://yourdomain.com" \
  -H "Access-Control-Request-Method: POST" \
  http://localhost:3001/messages
```

---

### 📦 Dependencies Added

```toml
ammonia = "4.0"        # XSS sanitization
governor = "0.6"       # Rate limiting
dotenvy = "0.15"       # Environment variables
```

---

### 🚨 Security Notes

1. **Never commit `.env` to Git** - add to `.gitignore`
2. **Rotate SERVER_SECRET** periodically
3. **Test all 3 env vars are required** - server will panic if missing
4. **Use HTTPS in production** - both frontend and Redis connection
5. **Monitor rate limit metrics** - adjust 50/minute if needed
6. **Keep cargo dependencies updated** - run `cargo audit`

---

### 📚 Files Modified

- ✅ `Cargo.toml` - Added ammonia, governor, dotenvy
- ✅ `server/.env` - Added DATABASE_URL, ALLOWED_ORIGIN
- ✅ `server/src/main.rs` - Updated to dotenvy, CORS config
- ✅ `server/src/models.rs` - Added sanitize_html()
- ✅ `server/src/security/governor_rate_limiter.rs` - Created
- ✅ `server/src/security/mod.rs` - Added GovernorRateLimiter
- ✅ `server/src/state.rs` - Added governor_limiter
- ✅ `server/src/security/middleware.rs` - Integrated governor checks
