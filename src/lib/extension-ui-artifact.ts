import type {
  ExtensionUiArtifact,
  ExtensionUiArtifactOutput,
} from "@/types/extensions";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";

const MAX_PATH_CHARS = 4_096;
const MAX_INPUTS = 8_192;
const ARTIFACT_TYPES = ["javascript", "css", "png", "jpeg", "webp", "gif", "woff2"] as const;
const SHA256_PATTERN = /^[0-9a-f]{64}$/u;
const BUILDER_VERSION_PATTERN = /^\d+\.\d+\.\d+$/u;
const NODE_VERSION_PATTERN = /^v?\d+\.\d+\.\d+$/u;
const ARTIFACT_NAME_PATTERN = /^[A-Za-z0-9_.-]{1,160}$/u;

export function parseExtensionUiArtifact(value: unknown): ExtensionUiArtifact | undefined {
  if (value === null || value === undefined) return undefined;
  const input = object(value);
  if (input.version !== 1
    || !Array.isArray(input.outputs)
    || !Array.isArray(input.inputs)
    || input.outputs.length === 0
    || input.outputs.length > UI_LIMITS.maxAdvancedArtifactFiles
    || input.inputs.length === 0
    || input.inputs.length > MAX_INPUTS
    || !Number.isSafeInteger(input.totalBytes)
    || (input.totalBytes as number) < 0
    || (input.totalBytes as number) > UI_LIMITS.maxAdvancedArtifactBytes) invalid();
  const builderVersion = text(input.builderVersion, 32);
  const nodeVersion = text(input.nodeVersion, 32);
  const entry = text(input.entry, 160);
  const manifestSha256 = text(input.manifestSha256, 64);
  if (!BUILDER_VERSION_PATTERN.test(builderVersion)
    || !NODE_VERSION_PATTERN.test(nodeVersion)
    || !ARTIFACT_NAME_PATTERN.test(entry)
    || !SHA256_PATTERN.test(manifestSha256)) invalid();
  const outputs = input.outputs.map(artifactOutput);
  const inputs = input.inputs.map(artifactInput);
  const outputNames = outputs.map(({ name }) => name);
  const entryOutput = outputs.find(({ name }) => name === entry);
  if (!sortedUnique(outputNames)
    || !sortedUnique(inputs)
    || entryOutput?.type !== "javascript"
    || outputs.filter(({ type }) => type === "javascript").length !== 1
    || outputs.reduce((sum, output) => sum + output.bytes, 0) !== input.totalBytes) invalid();
  return {
    version: 1,
    builderVersion,
    nodeVersion,
    entry,
    totalBytes: input.totalBytes,
    outputs,
    inputs,
    manifestSha256,
  };
}

function artifactOutput(value: unknown): ExtensionUiArtifactOutput {
  const input = object(value);
  const name = text(input.name, 160);
  const bytes = input.bytes;
  const sha256 = text(input.sha256, 64);
  const type = oneOf(input.type, ARTIFACT_TYPES);
  if (!ARTIFACT_NAME_PATTERN.test(name)
    || !Number.isSafeInteger(bytes)
    || (bytes as number) < 0
    || (bytes as number) > UI_LIMITS.maxAdvancedArtifactBytes
    || !artifactExtensionMatches(name, type)
    || !SHA256_PATTERN.test(sha256)) invalid();
  return { name, type, bytes: bytes as number, sha256 };
}

function artifactInput(value: unknown): string {
  const input = text(value, MAX_PATH_CHARS);
  const segments = input.split("/");
  if (input.startsWith("/")
    || input.includes("\\")
    || segments.some((segment) => !segment || segment === "." || segment === "..")) invalid();
  return input;
}

function artifactExtensionMatches(
  name: string,
  type: ExtensionUiArtifactOutput["type"],
): boolean {
  const extension = name.split(".").pop()?.toLowerCase();
  if (type === "javascript") return extension === "js" || extension === "mjs";
  if (type === "jpeg") return extension === "jpg" || extension === "jpeg";
  return extension === type;
}

function sortedUnique(values: readonly string[]): boolean {
  return values.every((value, index) => index === 0 || values[index - 1] < value);
}

function object(value: unknown): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) invalid();
  return value as Record<string, unknown>;
}

function text(value: unknown, maxChars: number): string {
  if (typeof value !== "string" || value.length > maxChars * 2) invalid();
  const length = Array.from(value).length;
  if (length === 0 || length > maxChars) invalid();
  return value;
}

function oneOf<T extends string>(value: unknown, values: readonly T[]): T {
  if (typeof value !== "string" || !values.includes(value as T)) invalid();
  return value as T;
}

function invalid(): never {
  throw new Error("invalid_extension_response");
}
