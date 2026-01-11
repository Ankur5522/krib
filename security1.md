To make this work effectively, you need to give your AI Copilot a prompt that combines data structure design (Redis) with logic flow (Rust/Axum).

Here is the professional-grade prompt to implement the "Progressive Friction" security layer:

The "IP Reputation & Progressive Friction" Backend Prompt
Context: I am building a "Zero-Friction" rental chat in Rust (Axum 0.7) using Redis for state. I need to implement a security layer that tracks IP reputation based on user reports.

The Goal: Prevent a single IP from being used for mass-scams by increasing "friction" (post delays) as more unique device fingerprints on that IP are reported.

Logic Requirements:

Data Tracking: When a post is reported, store the Fingerprint_ID in a Redis Set keyed by the User_IP (e.g., reports:ip:[IP_ADDRESS]). This allows us to count unique offenders on one IP using SCARD.

Risk Levels: Create an enum or constant for the following Risk Levels:

Level 0 (0-1 unique reports): 60s cooldown. Full broadcast.

Level 1 (2 unique reports): 300s (5m) cooldown. Full broadcast.

Level 2 (3-5 unique reports): 900s (15m) cooldown. Shadow-throttle: Broadcast only to the sender’s IP.

Level 3 (6+ unique reports): 7200s (2h) cooldown. Hard Shadowban: No broadcast.

The Middleware/Service: Create a Rust function get_ip_risk_level(ip: &str) -> RiskLevel that:

Queries Redis for the number of unique reported fingerprints on that IP.

Returns the corresponding cooldown time and "Visibility Mode" (Normal, Throttled, or Banned).

The Post Handler: In the message posting logic:

Check the get_ip_risk_level.

If the user is still in cooldown (check another Redis key cooldown:[CompositeKey]), return a generic "System busy, try again in X seconds" error.

If valid, update the cooldown timer and proceed with the broadcast logic based on the visibility mode.

Task: Write the Rust code for the RiskLevel enum, the Redis logic to increment reports, and the logic to calculate the current cooldown time. Ensure it uses tokio for async Redis calls.
