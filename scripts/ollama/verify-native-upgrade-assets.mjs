import { createHash } from "node:crypto";
import { mkdtemp, open, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const PLATFORM_KEYS = ["macos-aarch64", "linux-x64", "windows-x64"];
const PLATFORM_SUFFIXES = {
  "macos-aarch64": "aarch64.dmg",
  "linux-x64": "amd64.deb",
  "windows-x64": "x64-setup.exe",
};
const MAX_ASSET_BYTES = 1024 * 1024 * 1024;
const MAX_REDIRECTS = 3;
const RELEASE_HOST = "github.com";
const MANIFEST_FILE = join(dirname(fileURLToPath(import.meta.url)), "native-upgrade-assets.json");

function invalidManifest() {
  return new Error("Native upgrade asset manifest is invalid");
}

function hasExactKeys(value, keys) {
  return value && !Array.isArray(value) && typeof value === "object" &&
    Object.keys(value).length === keys.length && keys.every((key) => Object.hasOwn(value, key));
}

function requireObject(value) {
  if (!value || Array.isArray(value) || typeof value !== "object") throw invalidManifest();
  return value;
}

function validateAsset(platform, asset, version) {
  if (!hasExactKeys(asset, ["name", "url", "size", "sha256"])) throw invalidManifest();
  const expectedName = `Beaver_${version}_${PLATFORM_SUFFIXES[platform]}`;
  if (asset.name !== expectedName || basename(asset.name) !== asset.name) throw invalidManifest();
  if (!Number.isSafeInteger(asset.size) || asset.size <= 0 || asset.size > MAX_ASSET_BYTES) throw invalidManifest();
  if (typeof asset.sha256 !== "string" || !/^[0-9a-f]{64}$/.test(asset.sha256)) throw invalidManifest();

  let parsed;
  try {
    parsed = new URL(asset.url);
  } catch {
    throw invalidManifest();
  }
  const expectedPath = `/Kevin-hDev/Beaver/releases/download/v${version}/${expectedName}`;
  if (parsed.protocol !== "https:" || parsed.hostname !== RELEASE_HOST || parsed.port ||
      parsed.username || parsed.password || parsed.search || parsed.hash || parsed.pathname !== expectedPath) {
    throw invalidManifest();
  }
  if (asset.name.toUpperCase().includes("CL-GO-DASH")) throw invalidManifest();
}

export function verifyNativeUpgradeManifest(input) {
  const manifest = requireObject(input);
  if (!hasExactKeys(manifest, ["schemaVersion", "product", "fromVersion", "assets", "excluded"]) ||
      manifest.schemaVersion !== 1 || manifest.product !== "Beaver" || manifest.fromVersion !== "1.1.2") {
    throw invalidManifest();
  }
  const assets = requireObject(manifest.assets);
  if (Object.keys(assets).length !== PLATFORM_KEYS.length ||
      PLATFORM_KEYS.some((platform) => !Object.hasOwn(assets, platform))) throw invalidManifest();
  for (const platform of PLATFORM_KEYS) validateAsset(platform, assets[platform], manifest.fromVersion);

  if (!Array.isArray(manifest.excluded) || manifest.excluded.length !== 1) throw invalidManifest();
  const excluded = manifest.excluded[0];
  if (!hasExactKeys(excluded, ["product", "version", "reason"]) || excluded.product !== "CL-GO-DASH" ||
      excluded.version !== "1.0.2" || typeof excluded.reason !== "string" || excluded.reason.length === 0) {
    throw invalidManifest();
  }
  return { platforms: [...PLATFORM_KEYS], product: manifest.product, version: manifest.fromVersion };
}

export function constantTimeHexEqual(actual, expected) {
  if (typeof actual !== "string" || typeof expected !== "string") return false;
  const maxLength = Math.max(actual.length, expected.length);
  let difference = actual.length ^ expected.length;
  for (let index = 0; index < maxLength; index += 1) {
    const left = index < actual.length ? actual.charCodeAt(index) : 0;
    const right = index < expected.length ? expected.charCodeAt(index) : 0;
    difference |= left ^ right;
  }
  return difference === 0;
}

function assertReleaseUrl(value, assetName) {
  let parsed;
  try {
    parsed = new URL(value);
  } catch {
    throw new Error("Native upgrade asset verification failed");
  }
  if (parsed.protocol !== "https:" || parsed.hostname !== RELEASE_HOST || parsed.port ||
      parsed.username || parsed.password || parsed.search || parsed.hash || parsed.pathname.endsWith("/")) {
    throw new Error("Native upgrade asset verification failed");
  }
  if (basename(parsed.pathname) !== assetName) throw new Error("Native upgrade asset verification failed");
  return parsed.href;
}

async function downloadAndVerify(asset, fetchImpl, tempDirectory) {
  let requestUrl = assertReleaseUrl(asset.url, asset.name);
  for (let redirect = 0; redirect <= MAX_REDIRECTS; redirect += 1) {
    let response;
    try {
      response = await fetchImpl(requestUrl, { redirect: "manual" });
    } catch {
      throw new Error("Native upgrade asset verification failed");
    }
    const status = Number(response?.status);
    if (status >= 300 && status < 400) {
      const location = response.headers?.get?.("location");
      if (!location || redirect === MAX_REDIRECTS) throw new Error("Native upgrade asset verification failed");
      requestUrl = assertReleaseUrl(new URL(location, requestUrl).href, asset.name);
      continue;
    }
    if (status !== 200 || !response.body) throw new Error("Native upgrade asset verification failed");

    const destination = join(tempDirectory, asset.name);
    const handle = await open(destination, "w");
    let bytes = 0;
    const digest = createHash("sha256");
    try {
      for await (const chunk of response.body) {
        const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
        bytes += buffer.length;
        if (bytes > asset.size) throw new Error("Native upgrade asset verification failed");
        digest.update(buffer);
        await handle.write(buffer);
      }
    } finally {
      await handle.close();
    }
    const actualSha = digest.digest("hex");
    if (bytes !== asset.size || !constantTimeHexEqual(actualSha, asset.sha256)) {
      throw new Error("Native upgrade asset verification failed");
    }
    return { name: asset.name, size: bytes, sha256: actualSha };
  }
  throw new Error("Native upgrade asset verification failed");
}

export async function readNativeUpgradeManifest(manifestPath = MANIFEST_FILE) {
  let parsed;
  try {
    parsed = JSON.parse(await readFile(manifestPath, "utf8"));
  } catch {
    throw invalidManifest();
  }
  verifyNativeUpgradeManifest(parsed);
  return parsed;
}

export async function verifyNativeUpgradeAssets({
  manifest,
  manifestPath = MANIFEST_FILE,
  fetchImpl = globalThis.fetch,
  tempRoot = tmpdir(),
} = {}) {
  const source = manifest ?? await readNativeUpgradeManifest(manifestPath);
  verifyNativeUpgradeManifest(source);
  if (typeof fetchImpl !== "function") throw new Error("Native upgrade asset verification failed");
  const temporaryDirectory = await mkdtemp(join(tempRoot, "beaver-native-upgrade-"));
  try {
    const verified = [];
    for (const platform of PLATFORM_KEYS) {
      verified.push(await downloadAndVerify(source.assets[platform], fetchImpl, temporaryDirectory));
    }
    return { verified, platforms: [...PLATFORM_KEYS] };
  } finally {
    await rm(temporaryDirectory, { recursive: true, force: true });
  }
}

async function main() {
  const args = process.argv.slice(2);
  if (args.length === 1 && args[0] === "--manifest-only") {
    const manifest = await readNativeUpgradeManifest();
    process.stdout.write(`Verified ${Object.keys(manifest.assets).length} Beaver v1.1.2 assets; CL-GO-DASH 1.0.2 excluded.\n`);
    return;
  }
  if (args.length === 1 && args[0] === "--verify-downloads") {
    const result = await verifyNativeUpgradeAssets();
    process.stdout.write(`Verified ${result.verified.length} Beaver v1.1.2 assets.\n`);
    return;
  }
  throw new Error("Usage: --manifest-only or --verify-downloads");
}

if (fileURLToPath(import.meta.url) === process.argv[1]) {
  main().catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  });
}
