import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
export const repoRoot = resolve(scriptDir, "../..");
export const tauriDir = resolve(repoRoot, "src-tauri");

const MAX_MANIFEST_BYTES = 16_384;
const ALLOWED_KEYS = new Set(["darwin-arm64", "win32-x64"]);
const SHA256_PATTERN = /^[a-f0-9]{64}$/u;
const FILE_PATTERN = /^cef_binary_[a-zA-Z0-9+._-]{1,180}\.tar\.bz2$/u;
const TOOL_FILE_PATTERN = /^ninja-[a-z0-9.-]{1,80}\.zip$/u;

function readManifest(name) {
  const path = resolve(tauriDir, name);
  const raw = readFileSync(path, { encoding: "utf8", flag: "r" });
  if (Buffer.byteLength(raw) > MAX_MANIFEST_BYTES) {
    throw new Error("CEF artifact configuration is invalid");
  }
  return JSON.parse(raw);
}

function validateArtifact(key, artifact) {
  if (
    !ALLOWED_KEYS.has(key) ||
    typeof artifact !== "object" ||
    artifact === null
  ) {
    throw new Error("CEF artifact configuration is invalid");
  }
  if (
    !FILE_PATTERN.test(artifact.file) ||
    !SHA256_PATTERN.test(artifact.sha256)
  ) {
    throw new Error("CEF artifact configuration is invalid");
  }
  if (
    !Number.isSafeInteger(artifact.bytes) ||
    artifact.bytes < 1 ||
    artifact.bytes > 536_870_912
  ) {
    throw new Error("CEF artifact configuration is invalid");
  }
  if (artifact.root !== artifact.file.slice(0, -".tar.bz2".length)) {
    throw new Error("CEF artifact configuration is invalid");
  }
  const url = new URL(artifact.url);
  if (
    url.protocol !== "https:" ||
    url.hostname !== "cef-builds.spotifycdn.com"
  ) {
    throw new Error("CEF artifact configuration is invalid");
  }
  return Object.freeze({ ...artifact, allowedHosts: [url.hostname] });
}

export function selectedArtifact() {
  if (process.platform !== "darwin" && process.platform !== "win32")
    return null;
  const key = `${process.platform}-${process.arch}`;
  if (!ALLOWED_KEYS.has(key))
    throw new Error("CEF is unavailable on this architecture");

  const manifest = readManifest("cef-artifacts.json");
  const keys = Object.keys(manifest.artifacts ?? {});
  if (
    keys.length !== ALLOWED_KEYS.size ||
    keys.some((entry) => !ALLOWED_KEYS.has(entry))
  ) {
    throw new Error("CEF artifact configuration is invalid");
  }
  return validateArtifact(key, manifest.artifacts[key]);
}

export function selectedBuildTool() {
  if (process.platform !== "darwin" && process.platform !== "win32")
    return null;
  const key = `${process.platform}-${process.arch}`;
  if (!ALLOWED_KEYS.has(key))
    throw new Error("CEF is unavailable on this architecture");
  const tool = readManifest("cef-build-tools.json").tools?.[key];
  if (
    typeof tool !== "object" ||
    tool === null ||
    !TOOL_FILE_PATTERN.test(tool.file)
  ) {
    throw new Error("CEF build tool configuration is invalid");
  }
  if (
    !SHA256_PATTERN.test(tool.sha256) ||
    !Number.isSafeInteger(tool.bytes) ||
    tool.bytes < 1 ||
    tool.bytes > 16 * 1024 * 1024
  ) {
    throw new Error("CEF build tool configuration is invalid");
  }
  if (
    !["ninja", "ninja.exe"].includes(tool.entry) ||
    tool.entry !== tool.output
  ) {
    throw new Error("CEF build tool configuration is invalid");
  }
  const url = new URL(tool.url);
  if (url.protocol !== "https:" || url.hostname !== "github.com") {
    throw new Error("CEF build tool configuration is invalid");
  }
  return Object.freeze({
    ...tool,
    allowedHosts: ["github.com", "release-assets.githubusercontent.com"],
  });
}
