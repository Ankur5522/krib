Objective: Build a security middleware for a Rust (Axum 0.7) backend and React frontend that prevents spam, scraping, and scams without a login.

Layer 1: Identification (The "Composite ID")

Frontend: Integrate ThumbmarkJS to generate a high-entropy browser fingerprint.

Backend: On every request, generate a CompositeKey = Hash(User_IP + Browser_Fingerprint + Server_Secret). Use this key as the unique identifier for the user in Redis.

Layer 2: Rate Limiting (The "Pressure Valve")

Implement a Sliding Window rate limiter using Redis.

Post Limit: Allow 1 post every 60 seconds per CompositeKey.

Contact Reveal Limit: Allow only 5 "Click to Reveal Phone" actions per hour per CompositeKey to prevent scrapers.

Burst Protection: If an IP sends >20 requests in 2 seconds, block that IP globally for 30 minutes.

Layer 3: The Shadowban System (The "Ghost" Defense)

Create a shadowban flag in Redis for specific CompositeKeys.

Logic: If a user is shadowbanned, their WebSocket messages should "successfully" send from their perspective, but the server should refuse to broadcast them to other users. This prevents scammers from knowing they’ve been caught.

Layer 4: Content & Link Filtering (The "Bouncer")

Implement a regex-based filter to block messages containing:

Known scam URLs or "Telegram bot" links.

Hardcoded phone numbers inside the message body (force them to use the dedicated phone field).

High-frequency "copy-paste" phrases used by rental bots.

Layer 5: Scraper Defense (The "Honeypot")

Add a hidden "honey-pot" field in the message form that is invisible to humans (via CSS) but visible to bots. If this field is filled, immediately hard-block the CompositeKey.

Task: > 1. Write the Rust Axum Middleware that extracts the IP and Fingerprint to create the CompositeKey. 2. Write the Redis Logic to check for "Shadowban" status before processing any message. 3. Write the Rust logic for the "Click to Reveal" rate limiter.
