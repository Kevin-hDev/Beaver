import { open } from "@tauri-apps/plugin-shell";

const ALLOWED_HOSTS = new Set(["github.com", "huggingface.co", "pypi.org"]);
const MAX_SOURCE_URL_LENGTH = 512;

export function isAllowedForecastDevSource(value: string): boolean {
  if (!value || value.length > MAX_SOURCE_URL_LENGTH) return false;
  try {
    const url = new URL(value);
    return url.protocol === "https:"
      && !url.username
      && !url.password
      && !url.port
      && ALLOWED_HOSTS.has(url.hostname);
  } catch {
    return false;
  }
}

export async function openForecastDevSource(value: string): Promise<void> {
  if (!isAllowedForecastDevSource(value)) return;
  await open(value);
}
