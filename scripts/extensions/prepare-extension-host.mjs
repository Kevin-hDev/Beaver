import { spawn } from "node:child_process";
import { copyFile, mkdir, readdir, rm } from "node:fs/promises";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { prepareNodeRuntime } from "./node-runtime.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const sourceDirectory = resolve(root, "src-tauri/resources/extension-host");
const argumentsList = process.argv.slice(2);
if (
  argumentsList.length > 1
  || (argumentsList.length === 1 && argumentsList[0] !== "--dev")
) {
  throw new Error("Invalid extension host preparation mode");
}
const developmentOnly = argumentsList[0] === "--dev";
const hostDirectory = developmentOnly
  ? resolve(root, "src-tauri/target/extension-host")
  : sourceDirectory;

if (developmentOnly) {
  await resetDevelopmentHost();
  await copyHostSources(sourceDirectory, hostDirectory);
  await installProductionDependencies(hostDirectory);
} else {
  await installProductionDependencies(hostDirectory);
  await prepareNodeRuntime(hostDirectory);
}

async function resetDevelopmentHost() {
  const targetRoot = resolve(root, "src-tauri/target");
  if (
    dirname(hostDirectory) !== targetRoot
    || basename(hostDirectory) !== "extension-host"
  ) {
    throw new Error("Invalid development extension host directory");
  }
  await rm(hostDirectory, { recursive: true, force: true });
}

async function installProductionDependencies(directory) {
  await run(process.platform === "win32" ? "npm.cmd" : "npm", [
    "ci",
    "--ignore-scripts",
    "--omit=dev",
    "--omit=optional",
    "--prefix",
    directory,
  ]);
}

async function copyHostSources(source, destination) {
  const rootFiles = [
    "contract.json",
    "contract.mjs",
    "diagnostics.mjs",
    "extension-api.mjs",
    "host.mjs",
    "loader.mjs",
    "package-lock.json",
    "package.json",
    "protocol-output.mjs",
    "protocol.mjs",
    "versions.mjs",
  ];
  const sdkFiles = ["README.md", "index.d.ts", "index.mjs", "package.json"];
  await mkdir(resolve(destination, "sdk"), { recursive: true, mode: 0o700 });
  await Promise.all([
    ...rootFiles.map((file) =>
      copyFile(resolve(source, file), resolve(destination, file)),
    ),
    ...sdkFiles.map((file) =>
      copyFile(resolve(source, "sdk", file), resolve(destination, "sdk", file)),
    ),
  ]);
  await copyDirectory(
    resolve(source, "builtin-plugins"),
    resolve(destination, "builtin-plugins"),
    { count: 0 },
  );
}

async function copyDirectory(source, destination, state) {
  await mkdir(destination, { recursive: true, mode: 0o700 });
  const entries = await readdir(source, { withFileTypes: true });
  if (entries.length > 128) throw new Error("Too many bundled plugin entries");
  for (const entry of entries) {
    if (state.count >= 512) throw new Error("Too many bundled plugin files");
    state.count += 1;
    const from = resolve(source, entry.name);
    const to = resolve(destination, entry.name);
    if (entry.isDirectory()) await copyDirectory(from, to, state);
    else if (entry.isFile()) await copyFile(from, to);
    else throw new Error("Unsupported bundled plugin entry");
  }
}

function run(program, args) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(program, args, {
      cwd: root,
      shell: false,
      stdio: "inherit",
    });
    child.once("error", reject);
    child.once("close", (code) => {
      if (code === 0) resolvePromise();
      else reject(new Error("Extension host preparation failed"));
    });
  });
}
