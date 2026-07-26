import { spawnSync } from "node:child_process";
import {
  lstat,
  mkdir,
  opendir,
  rename,
  rm,
  rmdir,
  stat,
  writeFile,
} from "node:fs/promises";
import { posix, resolve } from "node:path";
import { randomBytes } from "node:crypto";
import { tauriDir } from "./cef-artifacts.mjs";

const MAX_ARCHIVE_ENTRIES = 20_000;
const MAX_LIST_BYTES = 16 * 1024 * 1024;
const MAX_LAYOUT_DEPTH = 16;
const LAYOUT_VERSION = Object.freeze({
  darwin: "source-v1",
  win32: "windows-flat-v1",
});

function runTar(args) {
  const result = spawnSync("tar", args, {
    encoding: "utf8",
    maxBuffer: MAX_LIST_BYTES,
    shell: false,
    timeout: 180_000,
    windowsHide: true,
  });
  if (result.status !== 0 || result.error)
    throw new Error("CEF extraction failed");
  return result.stdout;
}

function validateEntries(output, expectedRoot) {
  const entries = output.split(/\r?\n/u).filter(Boolean);
  if (entries.length === 0 || entries.length > MAX_ARCHIVE_ENTRIES) {
    throw new Error("CEF archive is invalid");
  }
  for (const entry of entries) {
    if (entry.length > 512 || entry.includes("\\") || posix.isAbsolute(entry)) {
      throw new Error("CEF archive is invalid");
    }
    const parts = entry.split("/").filter(Boolean);
    if (parts[0] !== expectedRoot || parts.some((part) => part === "..")) {
      throw new Error("CEF archive is invalid");
    }
  }
}

async function validateDirectoryTree(root, state, depth = 0) {
  if (depth > MAX_LAYOUT_DEPTH) throw new Error("CEF archive is invalid");
  const directory = await opendir(root);
  for await (const entry of directory) {
    state.entries += 1;
    if (state.entries > MAX_ARCHIVE_ENTRIES || entry.isSymbolicLink()) {
      throw new Error("CEF archive is invalid");
    }
    const path = resolve(root, entry.name);
    if (entry.isDirectory()) {
      await validateDirectoryTree(path, state, depth + 1);
    } else if (!entry.isFile()) {
      throw new Error("CEF archive is invalid");
    }
  }
}

async function moveDirectoryEntries(source, destination) {
  const directory = await opendir(source);
  for await (const entry of directory) {
    const target = resolve(destination, entry.name);
    try {
      await lstat(target);
      throw new Error("CEF archive is invalid");
    } catch (error) {
      if (error?.code !== "ENOENT") throw error;
    }
    await rename(resolve(source, entry.name), target);
  }
  await rmdir(source);
}

export async function normalizeWindowsCefLayout(root) {
  const release = resolve(root, "Release");
  const resources = resolve(root, "Resources");
  const releaseStat = await stat(release);
  const resourcesStat = await stat(resources);
  if (!releaseStat.isDirectory() || !resourcesStat.isDirectory()) {
    throw new Error("CEF archive is invalid");
  }
  await validateDirectoryTree(root, { entries: 0 });
  await moveDirectoryEntries(release, root);
  await moveDirectoryEntries(resources, root);
}

function expectedLayoutMarker(artifact, platform) {
  const version = LAYOUT_VERSION[platform];
  if (typeof version !== "string") throw new Error("CEF platform is invalid");
  return `${artifact.sha256}:${version}`;
}

export async function isCurrentCefLayout(
  current,
  artifact,
  platform = process.platform,
) {
  try {
    const marker = await import("node:fs/promises").then(({ readFile }) =>
      readFile(resolve(current, ".clgo-sha256"), "utf8"),
    );
    const cmake = await stat(resolve(current, "CMakeLists.txt"));
    return (
      marker.trim() === expectedLayoutMarker(artifact, platform) &&
      cmake.isFile()
    );
  } catch {
    return false;
  }
}

export async function extractVerifiedArchive(archive, artifact) {
  const base = resolve(tauriDir, ".cef-verified");
  const current = resolve(base, "current");
  if (await isCurrentCefLayout(current, artifact)) return current;

  validateEntries(runTar(["-tjf", archive]), artifact.root);
  const nonce = randomBytes(12).toString("hex");
  const temporary = resolve(base, `.extract-${nonce}`);
  const backup = resolve(base, `.previous-${nonce}`);
  await mkdir(temporary, { recursive: false, mode: 0o700 });
  try {
    runTar(["-xjf", archive, "-C", temporary]);
    const extracted = resolve(temporary, artifact.root);
    if (!(await stat(resolve(extracted, "CMakeLists.txt"))).isFile()) {
      throw new Error("CEF extraction failed");
    }
    if (process.platform === "win32") {
      await normalizeWindowsCefLayout(extracted);
    }
    await writeFile(
      resolve(extracted, ".clgo-sha256"),
      `${expectedLayoutMarker(artifact, process.platform)}\n`,
      { mode: 0o600 },
    );
    await writeFile(
      resolve(extracted, "archive.json"),
      `${JSON.stringify({ type: "minimal", name: artifact.root, sha1: "verified-by-sha256" }, null, 2)}\n`,
      { mode: 0o600 },
    );
    await rename(current, backup).catch(() => {});
    await rename(extracted, current);
    await writeFile(resolve(current, ".gitkeep"), "", { mode: 0o600 });
    await rm(backup, { recursive: true, force: true });
  } catch (error) {
    await rename(backup, current).catch(() => {});
    throw error;
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
  return current;
}
