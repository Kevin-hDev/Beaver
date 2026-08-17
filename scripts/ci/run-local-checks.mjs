import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const npm = process.platform === "win32" ? "npm.cmd" : "npm";
const cargo = process.platform === "win32" ? "cargo.exe" : "cargo";
const rustRoot = join(root, "src-tauri");
const timeout = 10 * 60 * 1000;
const rustEnv = {
  ...process.env,
  CARGO_INCREMENTAL: "0",
  CARGO_PROFILE_DEV_DEBUG: "0",
  CARGO_PROFILE_TEST_DEBUG: "0",
};

const checks = [
  ["TypeScript", process.execPath, ["node_modules/typescript/bin/tsc", "--noEmit"], root],
  ["Lint frontend", npm, ["run", "lint"], root],
  ["Build frontend", npm, ["run", "build"], root],
  ["Tests frontend et scripts", npm, ["test"], root],
  ["Format Rust", cargo, ["fmt", "--all", "--", "--check"], rustRoot, rustEnv],
  ["Clippy Rust", cargo, ["clippy", "--all-targets", "--", "-D", "warnings"], rustRoot, rustEnv],
  ["Tests Rust", cargo, ["test", "--all"], rustRoot, rustEnv],
];

if (process.argv.includes("--list")) {
  for (const [name] of checks) console.log(name);
  process.exit(0);
}

const results = [];
for (const [name, command, args, cwd, env = process.env] of checks) {
  console.log(`\n=== ${name} ===`);
  const started = Date.now();
  const result = spawnSync(command, args, {
    cwd,
    env,
    stdio: "inherit",
    shell: false,
    timeout,
    killSignal: "SIGTERM",
  });
  const timedOut = result.error?.code === "ETIMEDOUT";
  const passed = result.status === 0 && !result.error;
  if (result.error && !timedOut) console.error(result.error.message);
  results.push({ name, passed, timedOut, duration: Date.now() - started });
}

console.log("\n=== Résumé ===");
for (const { name, passed, timedOut, duration } of results) {
  const state = passed ? "VERT" : timedOut ? "TIMEOUT" : "ROUGE";
  console.log(`${state.padEnd(7)} ${name} (${Math.round(duration / 1000)} s)`);
}

const failures = results.filter(({ passed }) => !passed);
console.log(`\n${results.length - failures.length}/${results.length} contrôles verts.`);
process.exitCode = failures.length === 0 ? 0 : 1;
