import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

const DEFAULT_SESSION_NAMES = new Set(["Nouvelle session", "New session"]);

export function displaySessionName(name: string, t: (key: string) => string): string {
  return DEFAULT_SESSION_NAMES.has(name) ? t("agentLocal.newSession") : name;
}

/** Compare deux IDs non-secrets (UUIDs de session/tab) */
export function idMatch(a: string | null | undefined, b: string): boolean {
  if (!a) return false;
  return a.localeCompare(b) === 0;
}
