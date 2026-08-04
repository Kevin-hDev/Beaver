import { lstat, open, realpath } from "node:fs/promises";
import { isAbsolute, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const ERROR_MESSAGE = "Artifact path resolution failed";
const MAX_OUTPUT_BYTES = 1024 * 1024;
const MAX_OUTPUT_PATH_LENGTH = 4096;
const MAX_RESULT_LENGTH = 512;
const TAG_PATTERN = /^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u;
const TARGET_PATTERN = /^[A-Za-z0-9_.-]{1,128}$/u;
const BUNDLES = Object.freeze({
  dmg: "_aarch64.dmg",
  deb: "_amd64.deb",
  nsis: "_x64-setup.exe",
});

function fail() {
  throw new Error(ERROR_MESSAGE);
}

function safeText(value, maximumLength) {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    value.length <= maximumLength &&
    !/[\0\r\n]/u.test(value) &&
    !value.includes("..")
  );
}

export function resolveArtifact({ tag, target, bundleDir, suffix } = {}) {
  try {
    if (
      !safeText(tag, 64) ||
      !TAG_PATTERN.test(tag) ||
      !safeText(target, 128) ||
      !TARGET_PATTERN.test(target) ||
      !Object.hasOwn(BUNDLES, bundleDir) ||
      BUNDLES[bundleDir] !== suffix
    ) {
      fail();
    }
    const version = tag.slice(1);
    const base = `src-tauri/target/${target}/release/bundle`;
    const asset = `${base}/${bundleDir}/Beaver_${version}${suffix}`;
    if (asset.length > MAX_RESULT_LENGTH || asset.includes("..")) fail();
    if (bundleDir !== "dmg") return { asset };
    const app = `${base}/macos/Beaver.app`;
    if (app.length > MAX_RESULT_LENGTH) fail();
    return { asset, app };
  } catch {
    fail();
  }
}

function comparablePath(value) {
  const normalized = value
    .replace(/^\\\\\?\\UNC\\/iu, "\\\\")
    .replace(/^\\\\\?\\/u, "")
    .replaceAll("/", "\\");
  return process.platform === "win32" ? normalized.toLowerCase() : value;
}

function sameFile(left, right) {
  return left.dev === right.dev && left.ino === right.ino;
}

async function appendGithubOutput(outputPath, artifact) {
  let handle;
  try {
    if (
      !safeText(outputPath, MAX_OUTPUT_PATH_LENGTH) ||
      !isAbsolute(outputPath)
    ) {
      fail();
    }
    const before = await lstat(outputPath);
    const canonical = await realpath(outputPath);
    if (
      !before.isFile() ||
      before.isSymbolicLink() ||
      before.nlink > 1 ||
      before.size >= MAX_OUTPUT_BYTES ||
      comparablePath(canonical) !== comparablePath(outputPath)
    ) {
      fail();
    }
    const lines = [`asset=${artifact.asset}\n`];
    if (artifact.app) lines.push(`app=${artifact.app}\n`);
    const payload = lines.join("");
    if (Buffer.byteLength(payload, "utf8") > 2048) fail();
    handle = await open(canonical, "a");
    const opened = await handle.stat();
    const payloadBytes = Buffer.byteLength(payload, "utf8");
    if (
      !opened.isFile() ||
      !sameFile(before, opened) ||
      opened.size !== before.size ||
      opened.size + payloadBytes > MAX_OUTPUT_BYTES
    ) {
      fail();
    }
    await handle.writeFile(payload, "utf8");
    const after = await handle.stat();
    if (!sameFile(opened, after) || after.size !== opened.size + payloadBytes) fail();
    await handle.sync();
  } catch {
    fail();
  } finally {
    await handle?.close();
  }
}

async function main() {
  if (process.argv.length !== 2) fail();
  const artifact = resolveArtifact({
    tag: process.env.RELEASE_TAG,
    target: process.env.BUNDLE_TARGET,
    bundleDir: process.env.BUNDLE_DIR,
    suffix: process.env.ASSET_SUFFIX,
  });
  await appendGithubOutput(process.env.GITHUB_OUTPUT, artifact);
}

const invokedPath = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (import.meta.url === invokedPath) {
  main().catch(() => {
    process.stderr.write(`${ERROR_MESSAGE}\n`);
    process.exitCode = 1;
  });
}
