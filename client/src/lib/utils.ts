import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { v4 as uuidv4 } from "uuid";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const getDeviceId = (): string => {
  const STORAGE_KEY = "kirb_device_id";
  let deviceId = localStorage.getItem(STORAGE_KEY);
  if (!deviceId) {
    deviceId = uuidv4();
    localStorage.setItem(STORAGE_KEY, deviceId);
  }
  return deviceId;
};
