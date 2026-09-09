import { createHash } from "node:crypto";
import {
  closeSync,
  constants,
  fstatSync,
  lstatSync,
  openSync,
  readSync,
} from "node:fs";
import { lstat, open } from "node:fs/promises";

const READ_CHUNK_BYTES = 64 * 1024;
const MAX_SUPPORTED_BYTES = 2 * 1024 * 1024 * 1024;

function invalid() {
  return new Error("Regular file validation failed.");
}

function validateLimit(maxBytes) {
  if (!Number.isSafeInteger(maxBytes) || maxBytes < 1 || maxBytes > MAX_SUPPORTED_BYTES) {
    throw invalid();
  }
}

function validateOpened(opened, current, maxBytes, allowEmpty) {
  const minimum = allowEmpty ? 0n : 1n;
  if (
    !opened.isFile()
    || !current.isFile()
    || current.isSymbolicLink()
    || opened.dev !== current.dev
    || opened.ino !== current.ino
    || opened.size !== current.size
    || opened.size < minimum
    || opened.size > BigInt(maxBytes)
  ) {
    throw invalid();
  }
}

function sameSnapshot(left, right) {
  return (
    left.dev === right.dev
    && left.ino === right.ino
    && left.size === right.size
    && left.mtimeNs === right.mtimeNs
    && left.ctimeNs === right.ctimeNs
  );
}

function openFlags() {
  return constants.O_RDONLY | (process.platform === "win32" ? 0 : constants.O_NOFOLLOW);
}

export function withRegularFileSync(path, maxBytes, operation, options = {}) {
  validateLimit(maxBytes);
  let descriptor;
  try {
    descriptor = openSync(path, openFlags());
    const opened = fstatSync(descriptor, { bigint: true });
    validateOpened(opened, lstatSync(path, { bigint: true }), maxBytes, options.allowEmpty);
    const result = operation(descriptor, Number(opened.size));
    const after = fstatSync(descriptor, { bigint: true });
    const current = lstatSync(path, { bigint: true });
    validateOpened(after, current, maxBytes, options.allowEmpty);
    if (!sameSnapshot(opened, after)) throw invalid();
    return result;
  } finally {
    if (descriptor !== undefined) closeSync(descriptor);
  }
}

export function readRegularFileSync(path, maxBytes, options = {}) {
  return withRegularFileSync(path, maxBytes, (descriptor, size) => {
    const body = Buffer.allocUnsafe(size);
    let offset = 0;
    while (offset < size) {
      const bytesRead = readSync(descriptor, body, offset, size - offset, null);
      if (bytesRead === 0) throw invalid();
      offset += bytesRead;
    }
    const extra = Buffer.allocUnsafe(1);
    if (readSync(descriptor, extra, 0, 1, null) !== 0) throw invalid();
    return body;
  }, options);
}

export function readRegularTextSync(path, maxBytes, options = {}) {
  return readRegularFileSync(path, maxBytes, options).toString("utf8");
}

export async function withRegularFile(path, maxBytes, operation, options = {}) {
  validateLimit(maxBytes);
  let handle;
  try {
    handle = await open(path, openFlags());
    const opened = await handle.stat({ bigint: true });
    validateOpened(opened, await lstat(path, { bigint: true }), maxBytes, options.allowEmpty);
    const result = await operation(handle, Number(opened.size));
    const after = await handle.stat({ bigint: true });
    const current = await lstat(path, { bigint: true });
    validateOpened(after, current, maxBytes, options.allowEmpty);
    if (!sameSnapshot(opened, after)) throw invalid();
    return result;
  } finally {
    await handle?.close();
  }
}

export async function readRegularFile(path, maxBytes, options = {}) {
  return withRegularFile(path, maxBytes, async (handle, size) => {
    const body = Buffer.allocUnsafe(size);
    let offset = 0;
    while (offset < size) {
      const { bytesRead } = await handle.read(body, offset, size - offset, null);
      if (bytesRead === 0) throw invalid();
      offset += bytesRead;
    }
    const extra = Buffer.allocUnsafe(1);
    if ((await handle.read(extra, 0, 1, null)).bytesRead !== 0) throw invalid();
    return body;
  }, options);
}

export async function hashRegularFile(path, maxBytes) {
  return withRegularFile(path, maxBytes, async (handle, expectedSize) => {
    const hash = createHash("sha256");
    const buffer = Buffer.allocUnsafe(READ_CHUNK_BYTES);
    let size = 0;
    while (true) {
      const { bytesRead } = await handle.read(buffer, 0, buffer.length, null);
      if (bytesRead === 0) break;
      size += bytesRead;
      if (size > expectedSize || size > maxBytes) throw invalid();
      hash.update(buffer.subarray(0, bytesRead));
    }
    if (size !== expectedSize) throw invalid();
    return { sha256: hash.digest("hex"), size };
  });
}
