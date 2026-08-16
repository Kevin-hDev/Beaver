import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { lstat, readdir, realpath, rename, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { isAbsolute, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

export const MAX_INVENTORY_ENTRIES = 1000;
export const MAX_PROOF_BYTES = 4 * 1024 * 1024;
const DATA_DIRECTORY_SUFFIX = join(".local", "share", "cl-go-dash");
const MODELS_DIRECTORY_SUFFIX = join(".ollama", "models");
const ALLOWED_DATA_ROOTS = new Set([
  "ollama-bundle",
  "ollama-bundle-old",
  "ollama-bundle-backup",
  "ollama-bundle-failed",
  "ollama-bundle-staging",
  "ollama-bundle-update-staging",
  "ollama-bundle-install-staging",
  "ollama-process-receipt.json",
]);
const SENSITIVE_NAMES = new Set([
  "secrets.enc",
  "config.json",
  "configured-providers.json",
  "agent-settings.json",
  "api_keys.json",
  "master.key",
  ".env",
  ".env.local",
]);

function reject(message = "Native upgrade proof is invalid") {
  throw new Error(message);
}

function checkPathText(value) {
  if (typeof value !== "string" || value.length === 0 || value.includes("\0")) reject("Native upgrade proof path is invalid");
  if (value.split(/[\\/]+/u).includes("..")) reject("Native upgrade proof path is invalid");
}

async function realDirectory(path, label) {
  checkPathText(path);
  if (!isAbsolute(path)) reject(`Native upgrade proof ${label} is invalid`);
  const info = await lstat(path).catch(() => reject(`Native upgrade proof ${label} is invalid`));
  if (!info.isDirectory() || info.isSymbolicLink()) reject(`Native upgrade proof ${label} is invalid`);
  return realpath(path);
}

export async function canonicalizeDataDirectory({
  dataDirectory,
  modelsDirectory,
  homeDirectory = homedir(),
  confirmDisposableProfile = false,
  confirmDataDirectory = false,
} = {}) {
  if (confirmDisposableProfile !== true || confirmDataDirectory !== true) reject("Two explicit confirmations are required");
  const canonicalHome = await realDirectory(homeDirectory, "home directory");
  const canonicalData = await realDirectory(dataDirectory, "data directory");
  const canonicalModels = await realDirectory(modelsDirectory, "models directory");
  const expectedData = await realDirectory(join(canonicalHome, DATA_DIRECTORY_SUFFIX), "data directory");
  const expectedModels = await realDirectory(join(canonicalHome, MODELS_DIRECTORY_SUFFIX), "models directory");
  if (canonicalData !== expectedData || canonicalModels !== expectedModels) reject("Native upgrade proof directories are not the disposable profile");
  return {
    canonicalDataDirectory: canonicalData,
    canonicalModelsDirectory: canonicalModels,
    relativeDataDirectory: ".local/share/cl-go-dash",
    relativeModelsDirectory: ".ollama/models",
  };
}

async function hashFile(path, expectedBytes) {
  const digest = createHash("sha256");
  let bytes = 0;
  try {
    for await (const chunk of createReadStream(path, { highWaterMark: 64 * 1024 })) {
      bytes += chunk.length;
      digest.update(chunk);
    }
  } catch {
    reject("Native upgrade proof file cannot be read");
  }
  if (bytes !== expectedBytes) reject("Native upgrade proof file changed during collection");
  return digest.digest("hex");
}

async function collectContainedSymlink(path, scope, state, boundary) {
  const target = await realpath(path).catch(() => reject("Native upgrade proof symlink is not allowed"));
  const targetInfo = await lstat(target).catch(() => reject("Native upgrade proof symlink is not allowed"));
  if (!targetInfo.isFile()) reject("Native upgrade proof symlink is not allowed");
  let relativeTarget;
  try {
    safeRelativePath(boundary, target);
    relativeTarget = safeRelativePath(state.root, target);
  } catch {
    reject("Native upgrade proof symlink is not allowed");
  }
  state.count += 1;
  if (state.count > MAX_INVENTORY_ENTRIES) reject("Native upgrade proof has too many entries");
  state.entries.push({
    relativePath: `${scope}/${safeRelativePath(state.root, path)}`,
    symlinkTarget: `${scope}/${relativeTarget}`,
  });
}

function safeRelativePath(root, path) {
  const child = relative(root, path);
  if (!child || child.split(sep).includes("..") || child.startsWith(`..${sep}`) || isAbsolute(child)) {
    reject("Native upgrade proof traversal detected");
  }
  return child.split(sep).join("/");
}

async function collectDirectory(root, scope, state, boundary = root) {
  const entries = await readdir(root, { withFileTypes: true });
  entries.sort((left, right) => left.name.localeCompare(right.name));
  for (const entry of entries) {
    checkPathText(entry.name);
    const path = join(root, entry.name);
    const info = await lstat(path).catch(() => reject("Native upgrade proof entry cannot be inspected"));
    if (info.isSymbolicLink()) {
      await collectContainedSymlink(path, scope, state, boundary);
      continue;
    }
    if (info.isDirectory()) {
      await collectDirectory(path, scope, state, boundary);
      continue;
    }
    if (!info.isFile()) reject("Native upgrade proof entry type is not allowed");
    if (SENSITIVE_NAMES.has(entry.name.toLowerCase())) reject("Native upgrade proof contains a sensitive file");
    state.count += 1;
    if (state.count > MAX_INVENTORY_ENTRIES) reject("Native upgrade proof has too many entries");
    const relativePath = `${scope}/${safeRelativePath(state.root, path)}`;
    state.entries.push({ relativePath, bytes: info.size, sha256: await hashFile(path, info.size) });
  }
}

async function collectDataRoots(dataDirectory, state) {
  const entries = await readdir(dataDirectory, { withFileTypes: true });
  for (const entry of entries) {
    if (!ALLOWED_DATA_ROOTS.has(entry.name)) continue;
    const path = join(dataDirectory, entry.name);
    const info = await lstat(path).catch(() => reject("Native upgrade proof entry cannot be inspected"));
    if (info.isSymbolicLink()) reject("Native upgrade proof symlink is not allowed");
    if (info.isDirectory()) await collectDirectory(path, "data", state);
    else if (info.isFile()) {
      if (SENSITIVE_NAMES.has(entry.name.toLowerCase())) reject("Native upgrade proof contains a sensitive file");
      state.count += 1;
      if (state.count > MAX_INVENTORY_ENTRIES) reject("Native upgrade proof has too many entries");
      state.entries.push({ relativePath: `data/${entry.name}`, bytes: info.size, sha256: await hashFile(path, info.size) });
    } else reject("Native upgrade proof entry type is not allowed");
  }
}

export function serializeNativeUpgradeProof(proof) {
  if (!proof || !Array.isArray(proof.entries) || proof.entries.length > MAX_INVENTORY_ENTRIES) {
    reject("Native upgrade proof has too many entries");
  }
  const serialized = `${JSON.stringify(proof, null, 2)}\n`;
  if (Buffer.byteLength(serialized, "utf8") > MAX_PROOF_BYTES) reject("Native upgrade proof output size exceeds its limit");
  return serialized;
}

export async function collectNativeUpgradeProof(options = {}) {
  const directories = await canonicalizeDataDirectory(options);
  const state = { root: directories.canonicalDataDirectory, count: 0, entries: [] };
  await collectDataRoots(directories.canonicalDataDirectory, state);
  state.root = directories.canonicalModelsDirectory;
  await collectDirectory(directories.canonicalModelsDirectory, "models", state);
  const proof = {
    schemaVersion: 1,
    dataDirectory: directories.relativeDataDirectory,
    modelsDirectory: directories.relativeModelsDirectory,
    entries: state.entries,
  };
  serializeNativeUpgradeProof(proof);
  return proof;
}

export async function writeNativeUpgradeProof(proof, outputPath) {
  checkPathText(outputPath);
  if (!isAbsolute(outputPath) || SENSITIVE_NAMES.has(outputPath.split(/[\\/]+/u).pop().toLowerCase())) reject("Native upgrade proof output is invalid");
  const serialized = serializeNativeUpgradeProof(proof);
  const temporaryPath = `${outputPath}.tmp-${process.pid}`;
  await writeFile(temporaryPath, serialized, { encoding: "utf8", flag: "wx" });
  await rename(temporaryPath, outputPath);
}

function parseCli(args) {
  const values = {};
  const valueFlags = new Set(["--data-dir", "--models-dir", "--output"]);
  for (let index = 0; index < args.length; index += 1) {
    const flag = args[index];
    if (flag === "--confirm-disposable-profile" || flag === "--confirm-data-dir") {
      values[flag] = true;
      continue;
    }
    if (!valueFlags.has(flag) || values[flag] !== undefined || args[index + 1] === undefined) reject("Invalid native upgrade proof arguments");
    values[flag] = args[index + 1];
    index += 1;
  }
  if (!values["--data-dir"] || !values["--models-dir"] || !values["--confirm-disposable-profile"] || !values["--confirm-data-dir"]) {
    reject("Native upgrade proof requires both confirmations and explicit directories");
  }
  return values;
}

async function main() {
  const values = parseCli(process.argv.slice(2));
  const proof = await collectNativeUpgradeProof({
    dataDirectory: values["--data-dir"],
    modelsDirectory: values["--models-dir"],
    confirmDisposableProfile: true,
    confirmDataDirectory: true,
  });
  if (values["--output"]) await writeNativeUpgradeProof(proof, values["--output"]);
  else process.stdout.write(`${serializeNativeUpgradeProof(proof)}`);
}

if (fileURLToPath(import.meta.url) === process.argv[1]) {
  main().catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  });
}
