import { getFingerprint } from "@thumbmarkjs/thumbmarkjs";

let cachedFingerprint: string | null = null;

/**
 * Get browser fingerprint using ThumbmarkJS
 * The fingerprint is cached after the first call for performance
 *
 * @returns Promise resolving to the browser fingerprint string
 */
export async function getBrowserFingerprint(): Promise<string> {
  if (cachedFingerprint) {
    return cachedFingerprint;
  }

  try {
    const fingerprint = await getFingerprint();
    cachedFingerprint = fingerprint;
    return fingerprint;
  } catch (error) {
    console.error("Failed to generate browser fingerprint:", error);
    // Fallback to a simple hash if thumbmark fails
    cachedFingerprint = generateFallbackFingerprint();
    return cachedFingerprint;
  }
}

/**
 * Fallback fingerprint generator using basic browser properties
 * Used only if ThumbmarkJS fails
 */
function generateFallbackFingerprint(): string {
  const components = [
    navigator.userAgent,
    navigator.language,
    screen.width,
    screen.height,
    screen.colorDepth,
    new Date().getTimezoneOffset(),
    navigator.hardwareConcurrency || 0,
    navigator.maxTouchPoints || 0,
  ];

  const componentString = components.join("|");
  return simpleHash(componentString);
}

/**
 * Simple hash function for fallback fingerprinting
 */
function simpleHash(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash; // Convert to 32bit integer
  }
  return Math.abs(hash).toString(36);
}

/**
 * Clear the cached fingerprint (useful for testing)
 */
export function clearFingerprintCache(): void {
  cachedFingerprint = null;
}
