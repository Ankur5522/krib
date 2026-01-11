# Scalability Features - Quick Reference

## 🎯 New Endpoints

| Endpoint   | Method | Purpose                        | Response                                 |
| ---------- | ------ | ------------------------------ | ---------------------------------------- |
| `/health`  | GET    | Health check for load balancer | `200 OK` if healthy, `503` if Redis down |
| `/metrics` | GET    | Prometheus metrics             | Text format metrics for scraping         |

## 🔧 New Modules

### `server/src/scaling.rs`

- **RedisBroadcastService**: Handles Redis pub/sub for horizontal scaling
- **MetricsTracker**: Tracks Prometheus metrics (connections, messages, reveals)
- **HealthStatus**: Health check implementation

### `server/src/security/burst_profiler.rs`

- **BurstProfiler**: Detects bots hitting multiple endpoints rapidly (5 in 500ms)
- Uses Redis sorted sets for sliding window detection

## 📊 Prometheus Metrics

```promql
# Active WebSocket connections across all instances
active_websocket_connections

# Messages broadcast per second
rate(messages_per_second[1m])

# Contact reveals total
contact_reveals_total
```

## 🚨 Bot Detection

### Burst Profiler

- **Threshold**: 5 unique endpoints in 500ms
- **Action**: 24h shadowban + 30min IP block
- **Storage**: `burst:{composite_key}` in Redis

### Detection Flow

```
Request → Middleware extracts endpoint → Redis sorted set update →
Check unique count → If ≥5 in 500ms → SHADOWBAN + BLOCK IP
```

## 🔄 Horizontal Scaling Flow

```
User posts message → Server A receives →
Server A saves to Redis → Server A publishes to Redis pub/sub →
All servers (A, B, C) receive message →
All servers broadcast to their local WebSocket clients
```

## 🏥 Health Check Response

```json
{
  "healthy": true,
  "redis_connected": true,
  "active_connections": 42,
  "timestamp": 1736622547
}
```

## 🌐 Load Balancer Headers

The middleware extracts real IP from:

1. **Cloudflare**: `Cf-Connecting-Ip`
2. **Nginx/Standard**: `X-Forwarded-For` (first IP)
3. **Fallback**: Direct socket address

## 🛑 Graceful Shutdown

Signals handled:

- `CTRL+C` (SIGINT)
- `SIGTERM` (Docker/K8s)

Behavior:

- Stops accepting new connections
- Waits for pending WebSocket messages
- Cleanly closes Redis connections

## ⚙️ Configuration

### Environment Variables

```bash
REDIS_URL=redis://127.0.0.1:6379
SERVER_SECRET=your-secret-here
```

### Timeouts

- **Request timeout**: 30 seconds (tower-http)
- **Redis operations**: Default Redis client timeout
- **WebSocket idle**: No timeout (long-lived connections)

## 🔐 Security Enhancements

| Feature            | Implementation           | Location                     |
| ------------------ | ------------------------ | ---------------------------- |
| Real IP extraction | Middleware               | `security/middleware.rs`     |
| Burst detection    | Middleware + Redis       | `security/burst_profiler.rs` |
| Stateless auth     | CompositeKey per request | `security/middleware.rs`     |

## 📈 Capacity

Per instance (estimated):

- **Concurrent WebSockets**: 5,000+
- **Requests/second**: 10,000+ (with rate limiting)
- **Message latency**: <10ms (Redis pub/sub)

## 🐛 Debugging

### Check if server is healthy

```bash
curl http://localhost:3001/health
```

### View Prometheus metrics

```bash
curl http://localhost:3001/metrics | grep active_websocket_connections
```

### Check Redis pub/sub

```bash
redis-cli
> SUBSCRIBE chat:messages
# Should see messages being published
```

### Test burst detection

```bash
# Send 5 requests in <500ms
for i in {1..5}; do
  curl -H "X-Browser-Fingerprint: test" http://localhost:3001/api/cooldown &
done
wait
# Check server logs for 🤖 Bot detected
```

## 📦 Dependencies Added

```toml
metrics = "0.21"
metrics-exporter-prometheus = "0.13"
tokio = { features = ["signal"] }
tower-http = { features = ["timeout"] }
```

## ✅ Production Ready

All requirements from `scalability.md` implemented:

- ✅ Redis Pub/Sub for horizontal scaling
- ✅ Stateless architecture
- ✅ Health check endpoint
- ✅ Real IP extraction
- ✅ Burst profiler bot detection
- ✅ Prometheus metrics
- ✅ Graceful shutdown
- ✅ Timeout protection

---

**Ready to scale! 🚀**
