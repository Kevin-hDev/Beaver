import { randomBytes } from "node:crypto";
import { constants } from "node:fs";
import {
  lstat,
  open,
  realpath,
  rename,
  unlink,
} from "node:fs/promises";
import { basename, dirname, extname, isAbsolute, relative, resolve } from "node:path";
import { OFFICE_LIMITS } from "./constants.mjs";
import { rejectOffice } from "./errors.mjs";
import { requiredString } from "./validation.mjs";

export async function readWorkspaceFile(context, requestedPath, extensions) {
  const root = await workspaceRoot(context);
  const candidate = lexicalPath(root, requestedPath);
  let resolved;
  try {
    resolved = await realpath(candidate);
  } catch {
    rejectOffice("file_not_found");
  }
  ensureInside(root, resolved);
  validateExtension(resolved, extensions);
  const bytes = await readBoundedFile(resolved);
  return { bytes, path: resolved, root };
}

export async function workspaceOutput(context, requestedPath, extensions) {
  const root = await workspaceRoot(context);
  const candidate = lexicalPath(root, requestedPath);
  validateExtension(candidate, extensions);
  const parent = await realpath(dirname(candidate)).catch(() => null);
  if (!parent) rejectOffice("invalid_path");
  ensureInside(root, parent);
  const target = resolve(parent, basename(candidate));
  const metadata = await lstat(target).catch(() => null);
  if (metadata?.isSymbolicLink() || (metadata && !metadata.isFile())) {
    rejectOffice("invalid_path");
  }
  return { path: target, root };
}

export async function atomicWrite(outputPath, value) {
  const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value);
  if (bytes.length > OFFICE_LIMITS.maxOutputBytes) {
    bytes.fill(0);
    rejectOffice("output_too_large");
  }
  const nonce = randomBytes(16).toString("hex");
  const temporary = resolve(dirname(outputPath), `.${basename(outputPath)}.${nonce}.tmp`);
  let handle;
  try {
    handle = await open(temporary, "wx", 0o600);
    await handle.writeFile(bytes);
    await handle.sync();
    await handle.close();
    handle = undefined;
    await rename(temporary, outputPath);
  } catch {
    await handle?.close().catch(() => {});
    await unlink(temporary).catch(() => {});
    rejectOffice("operation_failed");
  } finally {
    bytes.fill(0);
  }
}

async function workspaceRoot(context) {
  const requested = requiredString(
    context?.workingDirectory,
    OFFICE_LIMITS.maxPathChars,
  );
  if (!isAbsolute(requested)) rejectOffice("invalid_path");
  const root = await realpath(requested).catch(() => null);
  if (!root) rejectOffice("invalid_path");
  const metadata = await lstat(root).catch(() => null);
  if (!metadata?.isDirectory()) rejectOffice("invalid_path");
  return root;
}

function lexicalPath(root, requestedPath) {
  const value = requiredString(requestedPath, OFFICE_LIMITS.maxPathChars);
  if (value.split(/[\\/]/u).some((part) => part === "..")) {
    rejectOffice("invalid_path");
  }
  const candidate = resolve(root, value);
  if (!isAbsolute(value)) ensureInside(root, candidate);
  return candidate;
}

async function readBoundedFile(path) {
  const noFollow = process.platform === "win32" ? 0 : constants.O_NOFOLLOW;
  let handle;
  let buffer;
  try {
    handle = await open(path, constants.O_RDONLY | noFollow);
    const metadata = await handle.stat();
    if (!metadata.isFile()) rejectOffice("invalid_path");
    if (metadata.size > OFFICE_LIMITS.maxInputBytes) {
      rejectOffice("file_too_large");
    }
    buffer = Buffer.allocUnsafe(metadata.size + 1);
    let offset = 0;
    while (offset < buffer.length) {
      const { bytesRead } = await handle.read(
        buffer,
        offset,
        buffer.length - offset,
        offset,
      );
      if (bytesRead === 0) break;
      offset += bytesRead;
    }
    if (offset > metadata.size) rejectOffice("file_too_large");
    return Buffer.from(buffer.subarray(0, offset));
  } catch (error) {
    if (error?.code === "ELOOP") rejectOffice("invalid_path");
    if (error?.code === "ENOENT") rejectOffice("file_not_found");
    if (error?.name === "OfficePluginError") throw error;
    rejectOffice("operation_failed");
  } finally {
    buffer?.fill(0);
    await handle?.close().catch(() => {});
  }
}

function ensureInside(root, candidate) {
  const child = relative(root, candidate);
  const separator = process.platform === "win32" ? "\\" : "/";
  if (
    child === ".."
    || child.startsWith(`..${separator}`)
    || isAbsolute(child)
  ) {
    rejectOffice("invalid_path");
  }
}

function validateExtension(path, extensions) {
  if (!extensions.includes(extname(path).toLowerCase())) {
    rejectOffice("unsupported_format");
  }
}
