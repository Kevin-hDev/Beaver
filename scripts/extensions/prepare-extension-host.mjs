import { spawn } from "node:child_process";
import { copyFile, mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";
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
  await copyHostSources(sourceDirectory, hostDirectory);
}

await run(process.platform === "win32" ? "npm.cmd" : "npm", [
  "ci",
  "--ignore-scripts",
  "--omit=dev",
  "--prefix",
  hostDirectory,
]);
if (!developmentOnly) {
  await prepareNodeRuntime(hostDirectory);
}

async function copyHostSources(source, destination) {
  const rootFiles = [
    "extension-api.mjs",
    "host.mjs",
    "loader.mjs",
    "package-lock.json",
    "package.json",
    "protocol.mjs",
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
