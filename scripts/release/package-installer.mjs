import { execFile as execFileCallback } from "node:child_process";
import { promisify } from "node:util";
import {
  chmod,
  copyFile,
  cp,
  lstat,
  mkdir,
  mkdtemp,
  open,
  opendir,
  realpath,
  rm,
  utimes,
  writeFile,
} from "node:fs/promises";
import { constants } from "node:fs";
import { basename, isAbsolute, join, relative, resolve, sep } from "node:path";
import { tmpdir } from "node:os";
import { pathToFileURL } from "node:url";

import {
  ASSET_SUFFIXES,
  expectedAssetName,
  normalizeVersion,
  resolveInputPath,
} from "./brand-artifact-common.mjs";

const execFile = promisify(execFileCallback);
const MAX_ENTRIES = 4_096;
const MAX_PATH_LENGTH = 1_024;
const FIXED_TIME = new Date("2000-01-01T00:00:00Z");
const CONFIG = Object.freeze({
  macOS: { kind: "macos_installer" },
  Windows: { kind: "windows_installer" },
});

function invalid(cause) {
  return new Error("Installer packaging failed.", { cause });
}

function safeRelative(path) {
  return (
    path.length > 0 &&
    path.length <= MAX_PATH_LENGTH &&
    !isAbsolute(path) &&
    !/[\0-\x1f\x7f]/u.test(path) &&
    !path.split(/[\\/]/u).includes("..")
  );
}

async function inventory(root, normalizeTimes = false) {
  const canonicalRoot = await realpath(root);
  const found = [];
  async function visit(path) {
    if (found.length >= MAX_ENTRIES) throw invalid();
    const info = await lstat(path);
    const rel = relative(canonicalRoot, path) || basename(canonicalRoot);
    if (!safeRelative(rel) || info.isSymbolicLink() || (!info.isDirectory() && !info.isFile())) {
      throw invalid();
    }
    if (info.isFile() && info.nlink !== 1) throw invalid();
    const canonical = await realpath(path);
    if (canonical !== canonicalRoot && !canonical.startsWith(`${canonicalRoot}${sep}`)) throw invalid();
    found.push({ path, rel, directory: info.isDirectory() });
    if (info.isDirectory()) {
      const entries = [];
      for await (const entry of await opendir(path)) entries.push(entry.name);
      entries.sort();
      for (const name of entries) await visit(join(path, name));
    }
    if (normalizeTimes) await utimes(path, FIXED_TIME, FIXED_TIME);
  }
  await visit(canonicalRoot);
  return found;
}

async function plistValue(plist, key) {
  const { stdout } = await execFile("plutil", ["-extract", key, "raw", "-o", "-", plist], {
    encoding: "utf8",
    maxBuffer: 4_096,
    timeout: 10_000,
  });
  return stdout.trim();
}

async function validateMacApp(app) {
  const canonical = await realpath(app);
  if (basename(canonical) !== "Beaver Installer.app") throw invalid();
  const items = await inventory(canonical);
  const plist = join(canonical, "Contents", "Info.plist");
  if ((await plistValue(plist, "CFBundleIdentifier")) !== "com.clgo.dash.installer") throw invalid();
  const executable = await plistValue(plist, "CFBundleExecutable");
  const binary = items.find(({ path }) => path === join(canonical, "Contents", "MacOS", executable));
  if (!binary || binary.directory) throw invalid();
}

async function validateWindowsExe(path) {
  const info = await lstat(path);
  if (!info.isFile() || info.isSymbolicLink() || info.nlink !== 1) throw invalid();
  const handle = await open(path, "r");
  try {
    const header = Buffer.alloc(2);
    const { bytesRead } = await handle.read(header, 0, 2, 0);
    if (bytesRead !== 2 || header[0] !== 0x4d || header[1] !== 0x5a) throw invalid();
  } finally {
    await handle.close();
  }
  if (process.platform === "win32") {
    const { stdout } = await execFile(
      "powershell.exe",
      ["-NoProfile", "-NonInteractive", "-Command", "(Get-Item -LiteralPath $args[0]).Attributes.value__", path],
      { encoding: "utf8", maxBuffer: 1_024, timeout: 10_000 },
    );
    if ((Number.parseInt(stdout.trim(), 10) & 0x400) !== 0) throw invalid();
  }
}

export function validateArchiveEntries(entries) {
  const names = entries.map((name) => name.replace(/\/$/u, ""));
  if (names.length < 1 || names.length > MAX_ENTRIES || new Set(names).size !== names.length) throw invalid();
  if (names.some((name) => !safeRelative(name) || (name !== "Beaver Installer.app" && !name.startsWith("Beaver Installer.app/")))) {
    throw invalid();
  }
}

async function validateArchive(archive, temporary) {
  const { stdout } = await execFile("tar", ["-tzf", archive], {
    encoding: "utf8",
    maxBuffer: 4 * 1024 * 1024,
    timeout: 30_000,
  });
  validateArchiveEntries(stdout.split("\n").filter(Boolean));
  const extracted = join(temporary, "verify");
  await mkdir(extracted);
  await execFile("tar", ["-xzf", archive, "-C", extracted, "--no-same-owner", "--no-same-permissions"], {
    timeout: 30_000,
  });
  await validateMacApp(join(extracted, "Beaver Installer.app"));
}

async function packageMac(source, destination, temporary) {
  await validateMacApp(source);
  const staging = join(temporary, "staging");
  const stagedApp = join(staging, "Beaver Installer.app");
  await mkdir(staging);
  await cp(source, stagedApp, { recursive: true, verbatimSymlinks: true });
  const entries = await inventory(stagedApp, true);
  const list = join(temporary, "entries.txt");
  await writeFile(list, `${entries.map(({ rel }) => join("Beaver Installer.app", rel === "Beaver Installer.app" ? "" : rel)).join("\n")}\n`, { flag: "wx" });
  const tar = join(temporary, "installer.tar");
  await execFile("tar", ["-cf", tar, "--format", "ustar", "--uid", "0", "--gid", "0", "--numeric-owner", "--no-recursion", "-C", staging, "-T", list], { timeout: 60_000 });
  await execFile("gzip", ["-n", "-9", tar], { timeout: 60_000 });
  const archive = `${tar}.gz`;
  await validateArchive(archive, temporary);
  await copyFile(archive, destination, constants.COPYFILE_EXCL);
}

export async function packageInstaller({ tag, os, source, suffix, outputDirectory = "." } = {}) {
  const config = CONFIG[os];
  if (!config || suffix !== ASSET_SUFFIXES[config.kind]) throw invalid();
  const version = normalizeVersion(tag);
  const sourcePath = resolveInputPath(source);
  const output = resolveInputPath(outputDirectory);
  const outputInfo = await lstat(output);
  if (!outputInfo.isDirectory() || outputInfo.isSymbolicLink()) throw invalid();
  const destination = join(output, expectedAssetName(config.kind, version));
  const temporary = await mkdtemp(join(tmpdir(), "beaver-package-installer-"));
  try {
    if (os === "macOS") await packageMac(sourcePath, destination, temporary);
    else {
      await validateWindowsExe(sourcePath);
      await copyFile(sourcePath, destination, constants.COPYFILE_EXCL);
      await chmod(destination, 0o700);
      await validateWindowsExe(destination);
    }
    return destination;
  } catch (error) {
    await rm(destination, { force: true }).catch(() => {});
    throw invalid(error);
  } finally {
    await rm(temporary, { recursive: true, force: true }).catch(() => {});
  }
}

async function main() {
  if (process.argv.length !== 5) throw invalid();
  const destination = await packageInstaller({
    tag: process.argv[2],
    os: process.argv[3],
    source: process.argv[4],
    suffix: process.env.INSTALLER_SUFFIX,
  });
  if (process.env.GITHUB_OUTPUT) {
    await writeFile(process.env.GITHUB_OUTPUT, `installer_asset=${destination}\n`, { flag: "a" });
  }
  process.stdout.write(`${destination}\n`);
}

const invoked = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (import.meta.url === invoked) main().catch(() => { process.stderr.write("Installer packaging failed.\n"); process.exitCode = 1; });
