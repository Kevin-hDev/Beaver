import type {
  ExtensionApiLevel,
  ExtensionContributions,
  ExtensionKind,
  ExtensionManifest,
  ExtensionOrigin,
  ExtensionOriginKind,
  ExtensionRecord,
  ExtensionResource,
  ExtensionSkill,
  ExtensionStatus,
  ExtensionTool,
} from "@/types/extensions";
import {
  EXTENSION_EFFECT_CLASSES,
  EXTENSION_EVENTS,
  EXTENSION_RESOURCE_TYPES,
  LIMITS,
  type ExtensionEffectClass,
  type ExtensionEvent,
  type ExtensionResourceType,
} from "@/types/extension-contract.generated";
import { EXTENSION_INSTALL_LIMITS } from "./extension-install";
import { parseExtensionManifestUi } from "./extension-manifest-ui";
import { parseExtensionUiArtifact } from "./extension-ui-artifact";
import {
  identifier,
  invalid,
  object,
  objectWithKeys,
  oneOf,
  optionalText,
  text,
} from "./extension-record-validation";

export { isExtensionIdentifier } from "./extension-record-validation";

export const EXTENSION_VIEW_LIMITS = Object.freeze({
  records: LIMITS.maxExtensions,
  toolsPerExtension: LIMITS.maxToolsPerExtension,
  eventsPerExtension: LIMITS.maxEventsPerExtension,
  skillsPerExtension: LIMITS.maxSkillsPerExtension,
  resourcesPerExtension: LIMITS.maxResourcesPerExtension,
});

const KINDS: readonly ExtensionKind[] = ["builtin", "local"];
const ORIGIN_KINDS: readonly ExtensionOriginKind[] = ["local", "git", "npm"];
const STATUSES: readonly ExtensionStatus[] = [
  "active",
  "inactive",
  "loading",
  "error",
  "incompatible",
];
const API_LEVELS: readonly ExtensionApiLevel[] = ["stable", "advanced"];

function skill(value: unknown): ExtensionSkill {
  const input = objectWithKeys(value, ["id", "name", "description", "path"]);
  return {
    id: identifier(input.id),
    name: text(input.name, LIMITS.maxExtensionNameChars),
    description: text(input.description, LIMITS.maxExtensionTextChars),
    path: text(input.path, LIMITS.maxPathChars),
  };
}

function resource(value: unknown): ExtensionResource {
  const input = objectWithKeys(value, ["id", "name", "description", "type", "path"]);
  return {
    id: identifier(input.id),
    name: text(input.name, LIMITS.maxExtensionNameChars),
    description: text(input.description, LIMITS.maxExtensionTextChars),
    type: oneOf(input.type, EXTENSION_RESOURCE_TYPES as readonly ExtensionResourceType[]),
    path: text(input.path, LIMITS.maxPathChars),
  };
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
    name: text(input.name, LIMITS.maxExtensionNameChars),
    version: text(input.version, 64),
    beaverApi: text(input.beaverApi, 16),
    runtime,
    main: optionalText(input.main, LIMITS.maxPathChars),
    ui: parseExtensionManifestUi(input.ui, LIMITS.maxPathChars),
    uiLegacy: optionalText(input.uiLegacy, LIMITS.maxPathChars),
    access,
    apiLevel: oneOf(input.apiLevel, API_LEVELS),
    essential: input.essential,
    author: optionalText(input.author, LIMITS.maxExtensionTextChars),
    homepage: optionalText(input.homepage, LIMITS.maxExtensionTextChars),
    description: optionalText(input.description, LIMITS.maxExtensionTextChars),
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
    description: text(input.description, LIMITS.maxExtensionTextChars),
    parameters: object(input.parameters),
    replacesCore: input.replacesCore,
    effect,
  };
}

function contributions(value: unknown): ExtensionContributions {
  const input = objectWithKeys(value, ["tools", "events", "skills", "resources"]);
  const skills = input.skills === undefined ? [] : input.skills;
  const resources = input.resources === undefined ? [] : input.resources;
  if (
    !Array.isArray(input.tools)
    || !Array.isArray(input.events)
    || !Array.isArray(skills)
    || !Array.isArray(resources)
    || input.tools.length > EXTENSION_VIEW_LIMITS.toolsPerExtension
    || input.events.length > EXTENSION_VIEW_LIMITS.eventsPerExtension
    || skills.length > EXTENSION_VIEW_LIMITS.skillsPerExtension
    || resources.length > EXTENSION_VIEW_LIMITS.resourcesPerExtension
  ) {
    invalid();
  }
  return {
    tools: input.tools.map(tool),
    events: input.events
      .map(identifier)
      .filter((event): event is ExtensionEvent =>
        EXTENSION_EVENTS.includes(event as ExtensionEvent)),
    skills: skills.map(skill),
    resources: resources.map(resource),
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
      kind === "local" ? LIMITS.maxPathChars : EXTENSION_INSTALL_LIMITS[kind],
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
  const parsedSource = text(input.source, LIMITS.maxPathChars);
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
    uiArtifact: parseExtensionUiArtifact(input.uiArtifact),
    showInChat: input.showInChat,
    status: oneOf(input.status, STATUSES),
    lastError: optionalText(input.lastError, LIMITS.maxExtensionTextChars),
    lastActivatedAt: optionalText(input.lastActivatedAt, 64),
    trustedAt: optionalText(input.trustedAt, 64),
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
