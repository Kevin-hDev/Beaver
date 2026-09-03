import { isExtensionIdentifier } from "@/lib/extension-records";
import type { ExtensionThemeChoice } from "@/lib/app-themes";
import {
  UI_THEME_TOKENS,
  UI_LIMITS,
  UI_VALIDATION,
} from "@/types/extension-ui-contract.generated";

const PUBLIC_TOKENS = new Set<string>(UI_THEME_TOKENS);
const MAX_THEME_CSS_BYTES = 1_048_576;
const MAX_RESOLUTION_DEPTH = 16;

export function extensionThemeChoice(id: string): ExtensionThemeChoice {
  if (!isExtensionIdentifier(id)) throw invalidTheme();
  return `extension:${id}`;
}

export function themeIdFromChoice(choice: ExtensionThemeChoice): string {
  const id = choice.slice("extension:".length);
  if (!isExtensionIdentifier(id)) throw invalidTheme();
  return id;
}

export function parseExtensionThemeTokens(value: unknown): Record<string, string> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw invalidTheme();
  const entries = Object.entries(value);
  if (entries.length > Math.min(UI_LIMITS.maxThemeTokens, UI_THEME_TOKENS.length)) {
    throw invalidTheme();
  }
  const tokens: Record<string, string> = {};
  for (const [name, color] of entries) {
    if (!PUBLIC_TOKENS.has(name) || typeof color !== "string" || !validExtensionColor(color)) {
      throw invalidTheme();
    }
    tokens[name] = color;
  }
  return tokens;
}

export function parseThemeCssDeclarations(css: string): ReadonlyMap<string, string> {
  if (new TextEncoder().encode(css).byteLength > MAX_THEME_CSS_BYTES) throw invalidTheme();
  const source = css.replace(/\/\*[\s\S]*?\*\//gu, "");
  const declarations = new Map<string, string>();
  const pattern = /(--[a-z0-9-]+)\s*:\s*([^;{}]+);/gu;
  for (const match of source.matchAll(pattern)) {
    const name = match[1];
    if (PUBLIC_TOKENS.has(name) && declarations.has(name)) throw invalidTheme();
    declarations.set(name, match[2].trim());
  }
  return declarations;
}

export function resolvePublicThemeColor(
  name: string,
  declarations: ReadonlyMap<string, string>,
): string {
  if (!PUBLIC_TOKENS.has(name)) throw invalidTheme();
  return resolveColor(name, declarations, new Set(), 0);
}

function resolveColor(
  name: string,
  declarations: ReadonlyMap<string, string>,
  visited: Set<string>,
  depth: number,
): string {
  if (depth > MAX_RESOLUTION_DEPTH || visited.has(name)) throw invalidTheme();
  const value = declarations.get(name);
  if (!value) throw invalidTheme();
  if (validBeaverColor(value)) return value;
  const reference = value.match(/^var\((--[a-z0-9-]+)\)$/u)?.[1];
  if (!reference) throw invalidTheme();
  const next = new Set(visited);
  next.add(name);
  return resolveColor(reference, declarations, next, depth + 1);
}

function validExtensionColor(value: string): boolean {
  return matchesHex(value)
    && UI_VALIDATION.themeValuePattern === "^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$";
}

function validBeaverColor(value: string): boolean {
  if (matchesHex(value)) return true;
  if (!value.startsWith("rgba(") || !value.endsWith(")")) return false;
  const channels = value.slice(5, -1).split(",").map((channel) => channel.trim());
  if (channels.length !== 4) return false;
  return channels.slice(0, 3).every((channel) => {
    return channel.length > 0
      && channel.length <= 3
      && Array.from(channel).every(isDecimalDigit)
      && Number(channel) <= 255;
  }) && validAlpha(channels[3]);
}

function matchesHex(value: string): boolean {
  return (value.length === 7 || value.length === 9)
    && value.startsWith("#")
    && Array.from(value.slice(1)).every((character) => {
      const code = character.charCodeAt(0);
      return (code >= 48 && code <= 57)
        || (code >= 65 && code <= 70)
        || (code >= 97 && code <= 102);
    });
}

function validAlpha(value: string): boolean {
  if (value === "0" || value === "1") return true;
  if (!(value.startsWith("0.") || value.startsWith("1."))) return false;
  const decimals = value.slice(2);
  return decimals.length > 0
    && Array.from(decimals).every(isDecimalDigit)
    && (value.startsWith("0.") || Array.from(decimals).every((digit) => digit === "0"));
}

function isDecimalDigit(character: string): boolean {
  const code = character.charCodeAt(0);
  return code >= 48 && code <= 57;
}

function invalidTheme(): Error {
  return new Error("invalid_extension_theme");
}
