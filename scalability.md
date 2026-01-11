ole: Senior DevOps & Rust Backend Architect. Objective: Refactor my Axum (0.7) backend for horizontal scaling, advanced observability, and high-tier bot defense.

1. Horizontal Scaling & Statelessness:

Redis Pub/Sub: Implement a RedisBroadcastService. Instead of storing WebSocket channels in a local HashMap, every message sent should be PUBLISHed to a Redis channel. Every server instance should SUBSCRIBE to that channel to broadcast messages to their locally connected users.

Stateless Auth: Ensure the CompositeKey (IP + Fingerprint) logic is entirely computed per request or stored in Redis with a TTL. No local memory caching.

2. Load Balancer Integration:

Health Checks: Add a /health endpoint that returns 200 OK only if the Redis connection is alive.

Real IP Extraction: Add middleware to correctly extract user IPs from X-Forwarded-For or Cf-Connecting-Ip headers (essential for Nginx/Cloudflare load balancing).

3. Advanced Bot Detection & Defense:

Behavioral Heuristics: Implement a "Burst Profiler" in Redis. If a single CompositeKey hits 5 different endpoints in under 500ms, flag them as a bot.

JA3/TLS Fingerprinting: If possible, integrate logic to inspect TLS handshakes (or use headers passed from the Load Balancer) to detect headless browsers (Puppeteer/Playwright).

4. Analytics & Metrics (Prometheus/Grafana):

Instrumentation: Use the metrics and axum-prometheus crates.

Custom Gauges: Track active_websocket_connections, messages_per_second, and contact_reveals_total.

Endpoint: Expose a protected /metrics endpoint for a Prometheus scraper.

5. Robustness & Error Handling:

Graceful Shutdown: Implement tokio::signal to ensure the server finishes sending pending messages before shutting down during a scaling event.

Timeout & Concurrency: Use tower_http::timeout::TimeoutLayer and ConcurrencyLimitLayer to prevent "Slowloris" attacks from exhausting server threads.

Task: Provide the updated main.rs architecture and a new scaling.rs module that handles the Redis Pub/Sub logic.
