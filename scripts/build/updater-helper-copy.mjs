import { createHash, randomBytes, timingSafeEqual } from "node:crypto";
import { lstat, mkdir, open, realpath, rename, rm } from "node:fs/promises";
import { dirname, isAbsolute, join, sep } from "node:path";

export const MAX_HELPER_BYTES = 64 * 1024 * 1024;
const ERROR_MESSAGE = "Updater helper preparation failed";
const MAX_PATH_LENGTH = 4096;
const COPY_CHUNK_BYTES = 1024 * 1024;
const TEMP_ATTEMPTS = 8;

function fail() {
  throw new Error(ERROR_MESSAGE);
}

function comparablePath(value) {
  const normalized = value.replaceAll("/", sep);
  return process.platform === "win32" ? normalized.toLowerCase() : normalized;
}

export function validateAbsolutePath(value) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.length > MAX_PATH_LENGTH ||
    /[\0\r\n]/u.test(value) ||
    !isAbsolute(value) ||
    value.split(/[\\/]+/u).includes("..")
  ) {
    fail();
  }
  return value;
}

export async function canonicalDirectory(path) {
  try {
    const info = await lstat(path);
    const canonical = await realpath(path);
    if (!info.isDirectory() || info.isSymbolicLink() || comparablePath(path) !== comparablePath(canonical)) fail();
    return canonical;
  } catch {
    fail();
  }
}

async function ensureDestinationParent(destination) {
  const parent = dirname(destination);
  let existing = parent;
  for (let depth = 0; depth < 64; depth += 1) {
    try {
      await lstat(existing);
      break;
    } catch (error) {
      if (error?.code !== "ENOENT") fail();
      const next = dirname(existing);
      if (next === existing) fail();
      existing = next;
    }
  }
  await canonicalDirectory(existing);
  try {
    await mkdir(parent, { recursive: true });
  } catch {
    fail();
  }
  return canonicalDirectory(parent);
}

function sameIdentity(first, second) {
  return (
    first.dev === second.dev &&
    first.ino === second.ino &&
    first.size === second.size &&
    first.nlink === second.nlink &&
    first.mode === second.mode &&
    first.ctimeMs === second.ctimeMs &&
    first.mtimeMs === second.mtimeMs
  );
}

async function checkedFile(path, maxBytes, maxLinks = 1) {
  try {
    const info = await lstat(path);
    const canonical = await realpath(path);
    if (
      !info.isFile() ||
      info.isSymbolicLink() ||
      info.nlink < 1 ||
      info.nlink > maxLinks ||
      info.size < 1 ||
      info.size > maxBytes ||
      comparablePath(path) !== comparablePath(canonical)
    ) {
      fail();
    }
    return info;
  } catch {
    fail();
  }
}

async function digestFile(path, maxBytes) {
  const before = await checkedFile(path, maxBytes);
  let handle;
  try {
    handle = await open(path, "r");
    const opened = await handle.stat();
    if (!sameIdentity(before, opened)) fail();
    const hash = createHash("sha256");
    const buffer = Buffer.allocUnsafe(COPY_CHUNK_BYTES);
    let total = 0;
    while (true) {
      const { bytesRead } = await handle.read(buffer, 0, buffer.length, null);
      if (bytesRead === 0) break;
      total += bytesRead;
      if (total > maxBytes) fail();
      hash.update(buffer.subarray(0, bytesRead));
    }
    const after = await handle.stat();
    if (total !== opened.size || !sameIdentity(opened, after)) fail();
    return hash.digest();
  } catch {
    fail();
  } finally {
    await handle?.close();
  }
}

async function copyToTemporary(source, temporary, maxBytes) {
  const before = await checkedFile(source, maxBytes, 2);
  let input;
  let output;
  try {
    input = await open(source, "r");
    output = await open(temporary, "wx", 0o700);
    const opened = await input.stat();
    if (!sameIdentity(before, opened)) fail();
    const hash = createHash("sha256");
    const buffer = Buffer.allocUnsafe(COPY_CHUNK_BYTES);
    let total = 0;
    while (true) {
      const { bytesRead } = await input.read(buffer, 0, buffer.length, null);
      if (bytesRead === 0) break;
      total += bytesRead;
      if (total > maxBytes) fail();
      hash.update(buffer.subarray(0, bytesRead));
      await output.write(buffer, 0, bytesRead, null);
    }
    const after = await input.stat();
    if (total !== opened.size || !sameIdentity(opened, after)) fail();
    await output.sync();
    await output.chmod(0o700);
    return hash.digest();
  } catch {
    fail();
  } finally {
    await input?.close();
    await output?.close();
  }
}

export async function copyVerifiedAtomic(source, destination, maxBytes = MAX_HELPER_BYTES) {
  let temporary;
  try {
    validateAbsolutePath(source);
    validateAbsolutePath(destination);
    if (!Number.isSafeInteger(maxBytes) || maxBytes < 1 || maxBytes > MAX_HELPER_BYTES || comparablePath(source) === comparablePath(destination)) fail();
    await checkedFile(source, maxBytes, 2);
    const parent = await ensureDestinationParent(destination);
    try {
      const existing = await lstat(destination);
      if (!existing.isFile() || existing.isSymbolicLink() || existing.nlink > 1) fail();
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
    for (let attempt = 0; attempt < TEMP_ATTEMPTS; attempt += 1) {
      temporary = join(parent, `.cl-go-dash-updater.${randomBytes(16).toString("hex")}.tmp`);
      try {
        const sourceDigest = await copyToTemporary(source, temporary, maxBytes);
        const copyDigest = await digestFile(temporary, maxBytes);
        if (!timingSafeEqual(sourceDigest, copyDigest)) fail();
        await rename(temporary, destination);
        temporary = undefined;
        return;
      } catch (error) {
        await rm(temporary, { force: true });
        temporary = undefined;
        if (error?.code !== "EEXIST") throw error;
      }
    }
    fail();
  } catch {
    fail();
  } finally {
    if (temporary) await rm(temporary, { force: true });
  }
}
