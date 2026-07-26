import { createHash, timingSafeEqual } from "node:crypto";
import { constants, lstat, open } from "node:fs/promises";
import { basename, relative, resolve, sep } from "node:path";

export const MAX_ASSET_BYTES = 2 * 1024 * 1024 * 1024;
export const MAX_MANIFEST_BYTES = 64 * 1024;
export const ASSET_SUFFIXES = Object.freeze({
  macos: "_aarch64.dmg",
  linux: "_amd64.deb",
  windows: "_x64-setup.exe",
});

const MAX_SOURCE_BYTES = 512 * 1024;
const MAX_ICON_BYTES = 8 * 1024 * 1024;
const MAX_U64 = 18_446_744_073_709_551_615n;
const VERSION_PATTERN = /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/u;
const ICONS = Object.freeze([
  ["icons/32x32.png", "89504e470d0a1a0a"],
  ["icons/128x128.png", "89504e470d0a1a0a"],
  ["icons/128x128@2x.png", "89504e470d0a1a0a"],
  ["icons/icon.icns", "69636e73"],
  ["icons/icon.ico", "00000100"],
]);

export function invalid() {
  return new Error("Beaver artifact validation failed.");
}

export function normalizeVersion(value) {
  if (typeof value !== "string" || value.length > 32) throw invalid();
  const version = value.startsWith("v") ? value.slice(1) : value;
  if (
    !VERSION_PATTERN.test(version) ||
    version.split(".").some((part) => BigInt(part) > MAX_U64)
  ) {
    throw invalid();
  }
  return version;
}

export function expectedAssetName(platform, versionValue) {
  const suffix = ASSET_SUFFIXES[platform];
  if (!suffix) throw invalid();
  return `Beaver_${normalizeVersion(versionValue)}${suffix}`;
}

export function resolveInputPath(value) {
  if (
    typeof value !== "string" ||
    value.length < 1 ||
    value.length > 4_096 ||
    /[\0-\x1f\x7f]/u.test(value) ||
    value.split(/[\\/]/u).includes("..")
  ) {
    throw invalid();
  }
  return resolve(value);
}

export function inside(root, path) {
  const result = resolve(root, path);
  const rel = relative(root, result);
  if (!rel || rel === ".." || rel.startsWith(`..${sep}`)) throw invalid();
  return result;
}

export async function metadata(path, maxBytes = MAX_ASSET_BYTES) {
  const value = await lstat(path);
  if (!value.isFile() || value.isSymbolicLink() || value.size < 1 || value.size > maxBytes) {
    throw invalid();
  }
  return value;
}

export async function readBounded(path, maxBytes) {
  const before = await metadata(path, maxBytes);
  const noFollow = process.platform === "win32" ? 0 : constants.O_NOFOLLOW;
  const handle = await open(path, constants.O_RDONLY | noFollow);
  try {
    const opened = await handle.stat();
    if (!opened.isFile() || opened.size !== before.size) throw invalid();
    return await handle.readFile();
  } finally {
    await handle.close();
  }
}

export async function readJson(path, maxBytes) {
  try {
    return JSON.parse((await readBounded(path, maxBytes)).toString("utf8"));
  } catch {
    throw invalid();
  }
}

export async function validateIcon(root, path, magic) {
  const body = await readBounded(inside(root, path), MAX_ICON_BYTES);
  if (!body.subarray(0, magic.length / 2).equals(Buffer.from(magic, "hex"))) throw invalid();
}

export async function validateSource(rootValue, versionValue) {
  const root = resolveInputPath(rootValue);
  const rootInfo = await lstat(root);
  if (!rootInfo.isDirectory() || rootInfo.isSymbolicLink()) throw invalid();
  const version = normalizeVersion(versionValue);
  const packageJson = await readJson(inside(root, "package.json"), MAX_SOURCE_BYTES);
  const config = await readJson(inside(root, "src-tauri/tauri.conf.json"), MAX_SOURCE_BYTES);
  const cargo = (await readBounded(
    inside(root, "src-tauri/Cargo.toml"),
    MAX_SOURCE_BYTES,
  )).toString("utf8");
  const cargoVersion = cargo.match(/^\[package\][\s\S]*?^version = "([^"]+)"$/mu)?.[1];
  if (
    packageJson.name !== "cl-go-dash" ||
    packageJson.version !== version ||
    cargoVersion !== version ||
    config.productName !== "Beaver" ||
    config.version !== version ||
    config.identifier !== "com.clgo.dash" ||
    JSON.stringify(config.bundle?.icon) !== JSON.stringify(ICONS.map(([path]) => path))
  ) {
    throw invalid();
  }
  await Promise.all(
    ICONS.map(([path, magic]) => validateIcon(inside(root, "src-tauri"), path, magic)),
  );
}

export async function validateAsset(platform, versionValue, pathValue) {
  const path = resolveInputPath(pathValue);
  if (basename(path) !== expectedAssetName(platform, versionValue)) throw invalid();
  await metadata(path);
  return path;
}

export async function hashFile(path) {
  const before = await metadata(path);
  const noFollow = process.platform === "win32" ? 0 : constants.O_NOFOLLOW;
  const handle = await open(path, constants.O_RDONLY | noFollow);
  try {
    const opened = await handle.stat();
    if (!opened.isFile() || opened.size !== before.size) throw invalid();
    const hash = createHash("sha256");
    let size = 0;
    for await (const chunk of handle.createReadStream({ autoClose: false })) {
      size += chunk.length;
      if (size > opened.size || size > MAX_ASSET_BYTES) throw invalid();
      hash.update(chunk);
    }
    if (size !== opened.size) throw invalid();
    return { size, sha256: hash.digest("hex") };
  } finally {
    await handle.close();
  }
}

export function hashesMatch(left, right) {
  if (!/^[a-f0-9]{64}$/u.test(left) || !/^[a-f0-9]{64}$/u.test(right)) return false;
  return timingSafeEqual(Buffer.from(left, "hex"), Buffer.from(right, "hex"));
}
