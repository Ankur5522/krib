// Generate random names for users based on their device ID
const adjectives = [
  "Swift",
  "Bright",
  "Quiet",
  "Kind",
  "Bold",
  "Calm",
  "Wise",
  "Lucky",
  "Happy",
  "Brave",
  "Smart",
  "Quick",
  "Sunny",
  "Cool",
  "Warm",
  "Fresh",
  "Gentle",
  "Noble",
  "Clever",
  "Eager",
];

const animals = [
  "Eagle",
  "Lion",
  "Wolf",
  "Bear",
  "Fox",
  "Hawk",
  "Tiger",
  "Panda",
  "Owl",
  "Deer",
  "Rabbit",
  "Dolphin",
  "Otter",
  "Seal",
  "Koala",
  "Lynx",
  "Falcon",
  "Raven",
  "Swan",
  "Crane",
];

export function generateRandomName(deviceId: string): string {
  // Use device ID as seed for consistent names
  let hash = 0;
  for (let i = 0; i < deviceId.length; i++) {
    hash = (hash << 5) - hash + deviceId.charCodeAt(i);
    hash = hash & hash; // Convert to 32bit integer
  }

  const adjIndex = Math.abs(hash) % adjectives.length;
  const animalIndex = Math.abs(hash >> 8) % animals.length;

  return `${adjectives[adjIndex]} ${animals[animalIndex]}`;
}
