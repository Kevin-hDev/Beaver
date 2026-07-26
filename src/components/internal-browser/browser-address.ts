import { MAX_BROWSER_URL_LENGTH, normalizeBrowserUrl } from "./browser-types";

const BROWSER_SEARCH_URL = "https://www.google.com/search";
const EXPLICIT_SCHEME = /^[a-z][a-z0-9+.-]*:/iu;

function byteLength(value: string): number {
  return new TextEncoder().encode(value).byteLength;
}

function containsControlCharacter(value: string): boolean {
  for (const character of value) {
    const codePoint = character.codePointAt(0);
    if (codePoint !== undefined && (codePoint <= 31 || codePoint === 127)) return true;
  }
  return false;
}

function isLoopbackAddress(input: string): boolean {
  try {
    const hostname = new URL(`http://${input}`).hostname.toLowerCase();
    const octets = hostname.split(".");
    return hostname === "localhost" || hostname === "[::1]" ||
      (octets.length === 4 && octets[0] === "127" && octets.every((part) => /^\d{1,3}$/u.test(part)));
  } catch {
    return false;
  }
}

function looksLikeHost(input: string): boolean {
  if (/\s/u.test(input)) return false;
  const authority = input.split(/[/?#]/u, 1)[0] ?? "";
  return authority.includes(".") || isLoopbackAddress(input);
}

export function resolveBrowserAddress(rawInput: string): string | null {
  if (
    !rawInput || byteLength(rawInput) > MAX_BROWSER_URL_LENGTH ||
    containsControlCharacter(rawInput) || rawInput.includes("\\")
  ) return null;
  const input = rawInput.trim();
  if (!input) return null;
  const directUrl = normalizeBrowserUrl(input);
  if (directUrl) return directUrl;
  const loopback = isLoopbackAddress(input);
  if (!loopback && !/\s/u.test(input) && EXPLICIT_SCHEME.test(input)) return null;
  if (looksLikeHost(input)) {
    const scheme = loopback ? "http" : "https";
    return normalizeBrowserUrl(`${scheme}://${input}`);
  }
  const search = new URL(BROWSER_SEARCH_URL);
  search.searchParams.set("q", input);
  return normalizeBrowserUrl(search.toString());
}
