import {
  lstat,
  mkdir,
  mkdtemp,
  open,
  readFile,
  rename,
  rm,
  unlink,
  writeFile,
} from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { copyDirectoryBounded } from "./runtime-copy.mjs";

const MANIFEST_NAME = "runtime-manifest.json";
const MAX_MANIFEST_BYTES = 4 * 1024;
const LOCK_WAIT_MS = 30_000;
const LOCK_POLL_MS = 100;

export async function ensureCachedRuntime(cacheRoot, key, descriptor, build) {
  validateCacheInput(cacheRoot, key, descriptor, build);
  await mkdir(cacheRoot, { recursive: true, mode: 0o700 });
  const cached = resolve(cacheRoot, key);
  if (await runtimeIsValid(cached, descriptor)) return cached;

  const lockPath = resolve(cacheRoot, `${key}.lock`);
  const lock = await acquireLock(lockPath, cached, descriptor);
  if (!lock) return cached;
  let staged;
  try {
    if (await runtimeIsValid(cached, descriptor)) return cached;
    staged = await mkdtemp(resolve(cacheRoot, `${key}.tmp-`));
    await build(staged);
    await writeFile(
      join(staged, MANIFEST_NAME),
      JSON.stringify(descriptor),
      { mode: 0o600 },
    );
    if (!(await runtimeIsValid(staged, descriptor))) {
      throw new Error("Prepared Node.js runtime is invalid");
    }
    await rm(cached, { recursive: true, force: true });
    await rename(staged, cached);
    staged = undefined;
    return cached;
  } finally {
    await lock.close();
    await unlink(lockPath).catch(() => {});
    if (staged) await rm(staged, { recursive: true, force: true });
  }
}

export async function materializeRuntime(cached, destination, descriptor) {
  if (await runtimeIsValid(destination, descriptor)) return;
  await mkdir(dirname(destination), { recursive: true, mode: 0o700 });
  const staged = await mkdtemp(resolve(dirname(destination), ".runtime-"));
  try {
    await copyDirectoryBounded(cached, staged);
    if (!(await runtimeIsValid(staged, descriptor))) {
      throw new Error("Cached Node.js runtime is invalid");
    }
    await rm(destination, { recursive: true, force: true });
    await rename(staged, destination);
  } catch (error) {
    await rm(staged, { recursive: true, force: true });
    throw error;
  }
}

export async function runtimeIsValid(directory, descriptor) {
  try {
    const manifestPath = join(directory, MANIFEST_NAME);
    const metadata = await lstat(manifestPath);
    if (!metadata.isFile() || metadata.size > MAX_MANIFEST_BYTES) return false;
    const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
    if (JSON.stringify(manifest) !== JSON.stringify(descriptor)) return false;
    const executable = join(
      directory,
      process.platform === "win32" ? "node.exe" : "node",
    );
    return await regularFile(executable)
      && await regularFile(join(directory, "npm/bin/npm-cli.js"))
      && await regularFile(join(directory, "NODE_LICENSE"));
  } catch {
    return false;
  }
}

async function regularFile(path) {
  const metadata = await lstat(path);
  return metadata.isFile() && !metadata.isSymbolicLink();
}

async function acquireLock(path, cached, descriptor) {
  const started = Date.now();
  while (Date.now() - started < LOCK_WAIT_MS) {
    try {
      return await open(path, "wx", 0o600);
    } catch (error) {
      if (error?.code !== "EEXIST") throw error;
      if (await runtimeIsValid(cached, descriptor)) return null;
      await new Promise((resolvePromise) => setTimeout(resolvePromise, LOCK_POLL_MS));
    }
  }
  throw new Error("Node.js runtime cache is busy");
}

function validateCacheInput(cacheRoot, key, descriptor, build) {
  if (
    typeof cacheRoot !== "string"
    || !/^[a-zA-Z0-9._-]{1,160}$/.test(key)
    || typeof build !== "function"
    || !descriptor
    || typeof descriptor !== "object"
    || Array.isArray(descriptor)
  ) {
    throw new Error("Invalid Node.js runtime cache configuration");
  }
}
