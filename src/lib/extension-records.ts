import type {
  ExtensionApiLevel,
  ExtensionContributions,
  ExtensionKind,
  ExtensionManifest,
  ExtensionOrigin,
  ExtensionOriginKind,
  ExtensionRecord,
  ExtensionStatus,
  ExtensionTool,
} from "@/types/extensions";
import {
  EXTENSION_EFFECT_CLASSES,
  EXTENSION_EVENTS,
  LIMITS,
  type ExtensionEffectClass,
  type ExtensionEvent,
} from "@/types/extension-contract.generated";
import { EXTENSION_INSTALL_LIMITS } from "./extension-install";

export const EXTENSION_VIEW_LIMITS = Object.freeze({
  records: LIMITS.maxExtensions,
  toolsPerExtension: LIMITS.maxToolsPerExtension,
  eventsPerExtension: LIMITS.maxEventsPerExtension,
});

const MAX_ID_CHARS = 96;
const MAX_NAME_CHARS = 100;
const MAX_TEXT_CHARS = 2_000;
const MAX_PATH_CHARS = 4_096;
const KINDS: readonly ExtensionKind[] = ["builtin", "local", "external"];
const ORIGIN_KINDS: readonly ExtensionOriginKind[] = ["local", "git", "npm"];
const STATUSES: readonly ExtensionStatus[] = [
  "active",
  "inactive",
  "loading",
  "error",
  "incompatible",
];
const API_LEVELS: readonly ExtensionApiLevel[] = ["stable", "advanced"];

function invalid(): never {
  throw new Error("invalid_extension_response");
}

function object(value: unknown): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    invalid();
  }
  return value as Record<string, unknown>;
}

function text(value: unknown, maxChars: number, allowEmpty = false): string {
  if (typeof value !== "string" || value.length > maxChars * 2) invalid();
  const length = Array.from(value).length;
  if (length > maxChars || (!allowEmpty && length === 0)) invalid();
  return value;
}

function optionalText(value: unknown, maxChars: number): string | undefined {
  return value === null || value === undefined
    ? undefined
    : text(value, maxChars);
}

function identifier(value: unknown): string {
  const parsed = text(value, MAX_ID_CHARS);
  const characters = Array.from(parsed);
  const valid = asciiAlphanumeric(characters[0])
    && asciiAlphanumeric(characters[characters.length - 1])
    && characters.every((character) =>
      asciiAlphanumeric(character) || [".", "_", "-"].includes(character));
  if (!valid) invalid();
  return parsed;
}

function asciiAlphanumeric(character: string | undefined): boolean {
  if (!character) return false;
  const code = character.charCodeAt(0);
  return (code >= 48 && code <= 57)
    || (code >= 65 && code <= 90)
    || (code >= 97 && code <= 122);
}

function oneOf<T extends string>(value: unknown, values: readonly T[]): T {
  if (typeof value !== "string" || !values.includes(value as T)) invalid();
  return value as T;
}

function manifest(value: unknown): ExtensionManifest {
  const input = object(value);
  const runtime = text(input.runtime, 32);
  const access = text(input.access, 16);
  if (!["node", "builtin"].includes(runtime) || !["full", "core"].includes(access)) {
    invalid();
  }
  if (typeof input.essential !== "boolean") invalid();
  return {
    id: identifier(input.id),
    name: text(input.name, MAX_NAME_CHARS),
    version: text(input.version, 64),
    beaverApi: text(input.beaverApi, 16),
    runtime,
    main: optionalText(input.main, MAX_PATH_CHARS),
    ui: optionalText(input.ui, MAX_PATH_CHARS),
    access,
    apiLevel: oneOf(input.apiLevel, API_LEVELS),
    essential: input.essential,
    author: optionalText(input.author, MAX_TEXT_CHARS),
    homepage: optionalText(input.homepage, MAX_TEXT_CHARS),
    description: optionalText(input.description, MAX_TEXT_CHARS),
  };
}

function tool(value: unknown): ExtensionTool {
  const input = object(value);
  if (typeof input.replacesCore !== "boolean") invalid();
  const effect = typeof input.effect === "string"
    && EXTENSION_EFFECT_CLASSES.includes(input.effect as ExtensionEffectClass)
    ? input.effect as ExtensionEffectClass
    : "unknown";
  return {
    name: identifier(input.name),
    description: text(input.description, MAX_TEXT_CHARS),
    parameters: object(input.parameters),
    replacesCore: input.replacesCore,
    effect,
  };
}

function contributions(value: unknown): ExtensionContributions {
  const input = object(value);
  if (
    !Array.isArray(input.tools)
    || !Array.isArray(input.events)
    || input.tools.length > EXTENSION_VIEW_LIMITS.toolsPerExtension
    || input.events.length > EXTENSION_VIEW_LIMITS.eventsPerExtension
  ) {
    invalid();
  }
  return {
    tools: input.tools.map(tool),
    events: input.events
      .map(identifier)
      .filter((event): event is ExtensionEvent =>
        EXTENSION_EVENTS.includes(event as ExtensionEvent)),
  };
}

function origin(value: unknown): ExtensionOrigin | undefined {
  if (value === null || value === undefined) return undefined;
  const input = object(value);
  const kind = oneOf(input.kind, ORIGIN_KINDS);
  const parsed = {
    kind,
    locator: text(
      input.locator,
      kind === "local" ? MAX_PATH_CHARS : EXTENSION_INSTALL_LIMITS[kind],
    ),
    revision: optionalText(input.revision, 128),
  };
  const validRevision = parsed.revision
    && [40, 64].includes(parsed.revision.length)
    && /^[0-9a-f]+$/.test(parsed.revision);
  if ((parsed.kind === "git" && !validRevision)
    || (parsed.kind !== "git" && parsed.revision)) invalid();
  return parsed;
}

function record(value: unknown): ExtensionRecord {
  const input = object(value);
  if (
    typeof input.enabled !== "boolean"
    || typeof input.trusted !== "boolean"
    || typeof input.showInChat !== "boolean"
  ) {
    invalid();
  }
  const parsedKind = oneOf(input.kind, KINDS);
  const parsedSource = text(input.source, MAX_PATH_CHARS);
  const parsedOrigin = origin(input.origin);
  if (parsedOrigin && parsedKind !== "local") invalid();
  if (parsedOrigin?.kind === "local" && parsedOrigin.locator !== parsedSource) invalid();
  return {
    manifest: manifest(input.manifest),
    kind: parsedKind,
    source: parsedSource,
    origin: parsedOrigin,
    enabled: input.enabled,
    trusted: input.trusted,
    showInChat: input.showInChat,
    status: oneOf(input.status, STATUSES),
    lastError: optionalText(input.lastError, MAX_TEXT_CHARS),
    lastActivatedAt: optionalText(input.lastActivatedAt, 64),
    contributions: contributions(input.contributions),
  };
}

export function parseExtensionRecords(value: unknown): ExtensionRecord[] {
  if (!Array.isArray(value) || value.length > EXTENSION_VIEW_LIMITS.records) {
    invalid();
  }
  const records = value.map(record);
  const identifiers = new Set(records.map((item) => item.manifest.id));
  if (identifiers.size !== records.length) invalid();
  return records;
}

export function parseExtensionRecord(value: unknown): ExtensionRecord {
  return record(value);
}
