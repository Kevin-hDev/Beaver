import { execFileSync, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const tauriDir = join(repoRoot, "src-tauri");
const commonGitDir = execFileSync(
  "git",
  ["rev-parse", "--path-format=absolute", "--git-common-dir"],
  { cwd: repoRoot, encoding: "utf8" },
).trim();
const sharedRepoRoot = dirname(commonGitDir);
const cefPath = join(sharedRepoRoot, "src-tauri", ".cef-verified", "current");
const cmakeWrapper = join(sharedRepoRoot, "src-tauri", ".cef-tools", "cmake-wrapper");

const args = [];
if (existsSync(join(cefPath, "archive.json"))) {
  args.push(
    "--config", `env.CEF_PATH.value=${JSON.stringify(cefPath)}`,
    "--config", "env.CEF_PATH.relative=false",
    "--config", "env.CEF_PATH.force=true",
  );
}
if (process.platform === "darwin" && existsSync(cmakeWrapper)) {
  args.push(
    "--config", `env.CMAKE_aarch64_apple_darwin.value=${JSON.stringify(cmakeWrapper)}`,
    "--config", "env.CMAKE_aarch64_apple_darwin.relative=false",
    "--config", "env.CMAKE_aarch64_apple_darwin.force=true",
  );
}
args.push(
  "test",
  "export_typescript_compression_profile_contract",
  "--",
  "--ignored",
  "--nocapture",
);

const result = spawnSync("cargo", args, {
  cwd: tauriDir,
  shell: false,
  stdio: "inherit",
  windowsHide: true,
});
process.exitCode = result.status ?? 1;
