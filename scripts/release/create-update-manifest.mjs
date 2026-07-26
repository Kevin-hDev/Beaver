import { createHash, randomBytes } from "node:crypto";
import {
  constants,
  lstat,
  open,
  opendir,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

export const MAX_UPDATE_ASSET_BYTES = 2 * 1024 * 1024 * 1024;
const MAX_DIRECTORY_ENTRIES = 16;
const MAX_U64 = 18_446_744_073_709_551_615n;
const MANIFEST_NAME = "update-manifest.json";
const VERSION_PATTERN = /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/;
const ASSET_SUFFIXES = [
  "_aarch64.dmg",
  "_amd64.deb",
  "_x64-setup.exe",
];

export function normalizeVersion(value) {
  if (typeof value !== "string" || value.length > 32) throw invalidManifest();
  const normalized = value.startsWith("v") ? value.slice(1) : value;
  if (!VERSION_PATTERN.test(normalized)) throw invalidManifest();
  if (normalized.split(".").some((part) => BigInt(part) > MAX_U64)) {
    throw invalidManifest();
  }
  return normalized;
}

export function isValidAssetSize(value) {
  return (
    Number.isSafeInteger(value) &&
    value >= 1 &&
    value <= MAX_UPDATE_ASSET_BYTES
  );
}

function expectedNames(version) {
  return new Set(ASSET_SUFFIXES.map((suffix) => `Beaver_${version}${suffix}`));
}

async function listExactAssets(directory, version) {
  const expected = expectedNames(version);
  const found = new Map();
  let entries = 0;
  const stream = await opendir(directory);
  for await (const entry of stream) {
    entries += 1;
    if (entries > MAX_DIRECTORY_ENTRIES) throw invalidManifest();
    if (entry.name === MANIFEST_NAME) continue;
    if (!expected.has(entry.name) || !entry.isFile() || found.has(entry.name)) {
      throw invalidManifest();
    }
    found.set(entry.name, resolve(directory, entry.name));
  }
  if (found.size !== expected.size) throw invalidManifest();
  return found;
}

async function hashAsset(path) {
  const before = await lstat(path);
  if (
    !before.isFile() ||
    before.isSymbolicLink() ||
    !isValidAssetSize(before.size)
  ) {
    throw invalidManifest();
  }
  const noFollow = process.platform === "win32" ? 0 : constants.O_NOFOLLOW;
  const handle = await open(path, constants.O_RDONLY | noFollow);
  try {
    const opened = await handle.stat();
    if (!opened.isFile() || opened.size !== before.size) throw invalidManifest();
    const hash = createHash("sha256");
    let bytes = 0;
    for await (const chunk of handle.createReadStream({ autoClose: false })) {
      bytes += chunk.length;
      if (bytes > opened.size || bytes > MAX_UPDATE_ASSET_BYTES) {
        throw invalidManifest();
      }
      hash.update(chunk);
    }
    if (bytes !== opened.size) throw invalidManifest();
    return { sha256: hash.digest("hex"), size: bytes };
  } finally {
    await handle.close();
  }
}

export async function createUpdateManifest(versionValue, directoryValue) {
  const version = normalizeVersion(versionValue);
  if (
    typeof directoryValue !== "string" ||
    directoryValue.length === 0 ||
    directoryValue.length > 4_096 ||
    /[\0-\x1f\x7f]/.test(directoryValue) ||
    directoryValue.split(/[\\/]/).includes("..")
  ) {
    throw invalidManifest();
  }
  const directory = resolve(directoryValue);
  const metadata = await lstat(directory);
  if (!metadata.isDirectory() || metadata.isSymbolicLink()) throw invalidManifest();
  const files = await listExactAssets(directory, version);
  const assets = [];
  for (const [name, path] of files) {
    assets.push({ name, ...(await hashAsset(path)) });
  }
  assets.sort((left, right) => (left.name < right.name ? -1 : left.name > right.name ? 1 : 0));
  return { version, assets };
}

export async function writeUpdateManifest(version, directory) {
  const manifest = await createUpdateManifest(version, directory);
  const destination = resolve(directory, MANIFEST_NAME);
  const temporary = resolve(
    directory,
    `.update-manifest-${randomBytes(16).toString("hex")}.tmp`,
  );
  try {
    await writeFile(temporary, `${JSON.stringify(manifest, null, 2)}\n`, {
      flag: "wx",
      mode: 0o600,
    });
    await rename(temporary, destination);
  } catch {
    await rm(temporary, { force: true }).catch(() => {});
    throw invalidManifest();
  }
  return destination;
}

function invalidManifest() {
  return new Error("Manifest de mise à jour invalide.");
}

const invokedPath = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (import.meta.url === invokedPath) {
  if (process.argv.length !== 4) {
    console.error("Usage : create-update-manifest <version> <dossier-assets>");
    process.exitCode = 1;
  } else {
    writeUpdateManifest(process.argv[2], process.argv[3])
      .then(() => console.log("update-manifest.json créé."))
      .catch(() => {
        console.error("Manifest de mise à jour invalide.");
        process.exitCode = 1;
      });
  }
}
