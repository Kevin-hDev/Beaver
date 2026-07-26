import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

const HISTORICAL_REPOSITORY = "Kevin-hDev/CL-GO-DASH";
const MAX_OUTPUT_BYTES = 512 * 1024;
const MAX_ASSET_BYTES = 2 * 1024 * 1024 * 1024;
const COMMAND_TIMEOUT_MS = 30_000;

function fail() {
  throw new Error("bridge publication failed");
}

function isRecord(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function normalizeContext(tagValue, repositoryValue) {
  if (
    repositoryValue !== HISTORICAL_REPOSITORY ||
    typeof tagValue !== "string" ||
    !/^v1\.0\.([2-9]|[1-9][0-9]{1,2})$/.test(tagValue)
  ) {
    fail();
  }
  return { repository: repositoryValue, tag: tagValue, version: tagValue.slice(1) };
}

function expectedAssetNames(version) {
  return new Set([
    `CL-GO_${version}_aarch64.dmg`,
    `CL-GO_${version}_amd64.deb`,
    `CL-GO_${version}_x64-setup.exe`,
  ]);
}

export function validateDraftRelease(value, tagValue) {
  const { version } = normalizeContext(tagValue, HISTORICAL_REPOSITORY);
  if (
    !isRecord(value) ||
    value.tagName !== tagValue ||
    value.name !== `CL-GO ${tagValue}` ||
    value.isDraft !== true ||
    value.isPrerelease !== false ||
    !Array.isArray(value.assets) ||
    value.assets.length !== 3
  ) {
    fail();
  }

  const expected = expectedAssetNames(version);
  const found = new Set();
  for (const asset of value.assets) {
    if (
      !isRecord(asset) ||
      typeof asset.name !== "string" ||
      !expected.has(asset.name) ||
      found.has(asset.name) ||
      asset.state !== "uploaded" ||
      !Number.isSafeInteger(asset.size) ||
      asset.size < 1 ||
      asset.size > MAX_ASSET_BYTES ||
      typeof asset.digest !== "string" ||
      !/^sha256:[a-f0-9]{64}$/.test(asset.digest)
    ) {
      fail();
    }
    found.add(asset.name);
  }
  if (found.size !== expected.size) fail();
}

function runGh(program, args, options) {
  const result = spawnSync(program, args, {
    encoding: "utf8",
    maxBuffer: options.maxOutputBytes,
    shell: false,
    stdio: ["ignore", "pipe", "pipe"],
    timeout: options.timeoutMs,
    windowsHide: true,
  });
  if (result.error || result.signal || result.status !== 0 || typeof result.stdout !== "string") {
    fail();
  }
  return result.stdout;
}

export function publishBridgeRelease({ tag, repository, run = runGh }) {
  const context = normalizeContext(tag, repository);
  const options = {
    maxOutputBytes: MAX_OUTPUT_BYTES,
    timeoutMs: COMMAND_TIMEOUT_MS,
  };
  const releaseJson = run(
    "gh",
    [
      "release",
      "view",
      context.tag,
      "--repo",
      context.repository,
      "--json",
      "tagName,name,isDraft,isPrerelease,assets",
    ],
    options,
  );

  let release;
  try {
    release = JSON.parse(releaseJson);
  } catch {
    fail();
  }
  validateDraftRelease(release, context.tag);

  run(
    "gh",
    [
      "release",
      "edit",
      context.tag,
      "--repo",
      context.repository,
      "--verify-tag",
      "--draft=false",
      "--latest",
    ],
    options,
  );
}

function isMainModule() {
  return Boolean(process.argv[1]) && pathToFileURL(process.argv[1]).href === import.meta.url;
}

if (isMainModule()) {
  try {
    if (process.argv.length !== 4) fail();
    publishBridgeRelease({
      tag: process.argv[2],
      repository: process.argv[3],
    });
  } catch {
    console.error("bridge publication failed");
    process.exitCode = 1;
  }
}
