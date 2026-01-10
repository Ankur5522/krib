# Frontend Security Integration Guide

## Step 1: Install ThumbmarkJS

```bash
cd client
pnpm install @thumbmarkjs/thumbmarkjs
```

## Step 2: Create a Fingerprint Hook

Create `client/src/hooks/useFingerprint.ts`:

```typescript
import { useState, useEffect } from "react";
import Thumbmark from "@thumbmarkjs/thumbmarkjs";

export function useFingerprint() {
  const [fingerprint, setFingerprint] = useState<string>("");
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function generateFingerprint() {
      try {
        const fp = await Thumbmark.get();
        setFingerprint(fp);
      } catch (error) {
        console.error("Failed to generate fingerprint:", error);
        // Fallback to a random ID if fingerprinting fails
        setFingerprint(`fallback_${Math.random().toString(36)}`);
      } finally {
        setIsLoading(false);
      }
    }

    generateFingerprint();
  }, []);

  return { fingerprint, isLoading };
}
```

## Step 3: Update API Calls

Modify your API calls to include the fingerprint header:

```typescript
// In your API utility or component
import { useFingerprint } from "./hooks/useFingerprint";

export function useApi() {
  const { fingerprint } = useFingerprint();

  const postMessage = async (data: PostMessageData) => {
    const response = await fetch("http://localhost:3001/messages", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Browser-Fingerprint": fingerprint,
      },
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Failed to post message");
    }

    return response.json();
  };

  const revealContact = async (messageId: string) => {
    const response = await fetch(
      `http://localhost:3001/api/contact/${messageId}`,
      {
        headers: {
          "X-Browser-Fingerprint": fingerprint,
        },
      }
    );

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || "Failed to reveal contact");
    }

    return response.json();
  };

  return { postMessage, revealContact, fingerprint };
}
```

## Step 4: Add Honeypot Field to Form

Update your message input form component:

```typescript
// In InputArea.tsx or your form component
<form onSubmit={handleSubmit}>
  {/* Regular form fields */}
  <textarea
    value={message}
    onChange={(e) => setMessage(e.target.value)}
    placeholder="Enter your message..."
  />

  <input
    type="tel"
    value={phone}
    onChange={(e) => setPhone(e.target.value)}
    placeholder="Phone number (optional)"
  />

  {/* Honeypot field - hidden from humans, visible to bots */}
  <input
    type="text"
    name="website"
    value={honeypot}
    onChange={(e) => setHoneypot(e.target.value)}
    style={{
      position: "absolute",
      left: "-9999px",
      width: "1px",
      height: "1px",
      opacity: 0,
    }}
    tabIndex={-1}
    autoComplete="off"
    aria-hidden="true"
  />

  <button type="submit">Post Message</button>
</form>
```

## Step 5: Handle Rate Limit Errors

Add error handling for rate limit responses:

```typescript
interface RateLimitError {
  error: string;
  retry_after: number;
}

interface ContentFilterError {
  error: string;
  reason: string;
}

async function postMessage(data: any) {
  try {
    const response = await fetch("http://localhost:3001/messages", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Browser-Fingerprint": fingerprint,
      },
      body: JSON.stringify({
        ...data,
        website: honeypot, // Include honeypot
      }),
    });

    if (response.status === 429) {
      const error: RateLimitError = await response.json();
      const retryDate = new Date(error.retry_after * 1000);
      const secondsRemaining = Math.ceil(
        (retryDate.getTime() - Date.now()) / 1000
      );

      throw new Error(
        `Rate limit exceeded. Please try again in ${secondsRemaining} seconds.`
      );
    }

    if (response.status === 403) {
      const error: ContentFilterError = await response.json();
      throw new Error(`Content violation: ${error.reason}`);
    }

    if (!response.ok) {
      throw new Error("Failed to post message");
    }

    return await response.json();
  } catch (error) {
    console.error("Post message error:", error);
    throw error;
  }
}
```

## Step 6: Update PostMessageRequest Interface

In your TypeScript types file:

```typescript
// client/src/types/index.ts
export interface PostMessageRequest {
  browser_id: string;
  message: string;
  message_type: "offered" | "requested";
  phone?: string;
  website?: string; // Honeypot field
}

export interface RateLimitError {
  error: string;
  retry_after: number;
}

export interface ContentFilterError {
  error: string;
  reason: string;
}
```

## Step 7: Add User Feedback

Display helpful error messages to users:

```typescript
function MessageForm() {
  const [error, setError] = useState<string>('');
  const [rateLimitSeconds, setRateLimitSeconds] = useState<number>(0);

  useEffect(() => {
    if (rateLimitSeconds > 0) {
      const timer = setInterval(() => {
        setRateLimitSeconds(prev => {
          if (prev <= 1) {
            setError('');
            return 0;
          }
          return prev - 1;
        });
      }, 1000);

      return () => clearInterval(timer);
    }
  }, [rateLimitSeconds]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    try {
      await postMessage({ ... });
    } catch (err) {
      if (err instanceof Error) {
        setError(err.message);

        // Extract seconds from rate limit message
        const match = err.message.match(/(\d+) seconds/);
        if (match) {
          setRateLimitSeconds(parseInt(match[1]));
        }
      }
    }
  };

  return (
    <div>
      {error && (
        <div className="error-banner">
          {error}
          {rateLimitSeconds > 0 && ` (${rateLimitSeconds}s remaining)`}
        </div>
      )}
      {/* Rest of form */}
    </div>
  );
}
```

## Testing the Integration

1. **Test Fingerprint Generation:**

   - Open browser console
   - Check that fingerprint is generated
   - Verify it's included in request headers

2. **Test Rate Limiting:**

   - Post multiple messages quickly
   - Should see rate limit error after 1st message
   - Wait 60 seconds, should work again

3. **Test Content Filtering:**

   - Try posting "Contact me on t.me/bot"
   - Should see content violation error

4. **Test Honeypot:**

   - Manually fill the hidden "website" field
   - Post message - should be blocked

5. **Test Contact Reveal Rate Limit:**
   - Click "Reveal Phone" more than 5 times in an hour
   - Should see rate limit error

## Important Notes

- The fingerprint should be generated once when the app loads
- Don't regenerate it on every request (it's expensive)
- The honeypot field must be completely hidden from users
- Handle rate limit errors gracefully with countdown timers
- Show user-friendly error messages for content violations
- Consider adding loading states while fingerprint generates
