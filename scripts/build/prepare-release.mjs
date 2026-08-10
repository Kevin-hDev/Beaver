import { lstat, realpath } from "node:fs/promises";
import { isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { runCommand } from "./command-runner.mjs";
import { buildFrontend as defaultBuildFrontend } from "./frontend-build.mjs";
import { prepareSearxng as defaultPrepareSearxng } from "./prepare-searxng.mjs";
import { prepareUpdaterHelper as defaultPrepareUpdater } from "./prepare-updater-helper.mjs";
import { canonicalDirectory } from "./updater-helper-copy.mjs";

const ERROR_MESSAGE = "Release preparation failed";
const PLATFORMS = new Set(["darwin", "linux", "win32"]);

function fail() {
  throw new Error(ERROR_MESSAGE);
}

async function canonicalFile(root, ...segments) {
  try {
    const expected = join(root, ...segments);
    const child = relative(root, expected);
    if (!child || child.startsWith("..") || isAbsolute(child)) fail();
    const [info, canonical] = await Promise.all([lstat(expected), realpath(expected)]);
    if (!info.isFile() || info.isSymbolicLink() || canonical !== expected) fail();
    return canonical;
  } catch {
    fail();
  }
}

function defaultPreparations(run) {
  return {
    async prepareExtensions({ repoRoot }) {
      const script = await canonicalFile(repoRoot, "scripts", "extensions", "prepare-extension-host.mjs");
      await run({ command: process.execPath, args: [script], cwd: repoRoot });
    },
    async prepareCefSource({ repoRoot }) {
      const script = await canonicalFile(repoRoot, "scripts", "cef", "prepare-cef-source.mjs");
      await run({ command: process.execPath, args: [script], cwd: repoRoot });
    },
    async buildFrontend({ repoRoot }) {
      await defaultBuildFrontend({ repoRoot, run });
    },
    async prepareUpdater({ repoRoot, platform }) {
      await defaultPrepareUpdater({
        platform,
        target: process.env.CARGO_BUILD_TARGET ?? "",
        tauriDir: join(repoRoot, "src-tauri"),
        cargoTargetDir: process.env.CARGO_TARGET_DIR ?? "",
        run,
      });
    },
    async prepareSearxng({ repoRoot }) {
      await defaultPrepareSearxng({ repoRoot, run });
    },
    async prepareUnixCef({ repoRoot }) {
      const script = await canonicalFile(repoRoot, "src-tauri", "scripts", "prepare-cef.sh");
      const tauriDir = await canonicalDirectory(join(repoRoot, "src-tauri"));
      await run({ command: "bash", args: [script], cwd: tauriDir });
    },
  };
}

export async function prepareRelease({
  repoRoot,
  platform = process.platform,
  prepareExtensions,
  prepareCefSource,
  buildFrontend,
  prepareUpdater,
  prepareSearxng,
  prepareUnixCef,
  run = runCommand,
} = {}) {
  try {
    if (!PLATFORMS.has(platform) || typeof run !== "function") fail();
    const root = await canonicalDirectory(repoRoot);
    const defaults = defaultPreparations(run);
    const selected = [
      prepareExtensions ?? defaults.prepareExtensions,
      prepareCefSource ?? defaults.prepareCefSource,
      buildFrontend ?? defaults.buildFrontend,
      prepareUpdater ?? defaults.prepareUpdater,
      prepareSearxng ?? defaults.prepareSearxng,
      prepareUnixCef ?? defaults.prepareUnixCef,
    ];
    if (!selected.every((preparation) => typeof preparation === "function")) fail();
    const context = Object.freeze({ repoRoot: root, platform });
    await selected[0](context);
    await selected[1](context);
    await selected[2](context);
    await selected[3](context);
    await selected[4](context);
    if (platform !== "win32") await selected[5](context);
  } catch {
    fail();
  }
}

const invokedPath = process.argv[1] ? pathToFileURL(resolve(process.argv[1])).href : "";
if (import.meta.url === invokedPath) {
  const repoRoot = resolve(fileURLToPath(new URL("../..", import.meta.url)));
  prepareRelease({ repoRoot }).catch(() => {
    process.stderr.write(`${ERROR_MESSAGE}\n`);
    process.exitCode = 1;
  });
}
