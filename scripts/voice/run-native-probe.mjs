import { spawn } from "node:child_process";
import { readFile, realpath, rename, rm, writeFile } from "node:fs/promises";
import { basename, dirname, isAbsolute, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { runCommand } from "../e2e/e2e-process.mjs";

const EXIT_TIMEOUT_MS = 30_000;
const MATRIX_TIMEOUT_MS = 55 * 60 * 1000;
const ERROR = "Voice native probe configuration is invalid";

export async function observeExit(binary, mode, {
  cwd = process.cwd(),
  timeoutMs = EXIT_TIMEOUT_MS,
  spawnProcess = spawn,
  evidencePath,
} = {}) {
  if (
    !isAbsolute(binary)
    || !["cooperative", "forced"].includes(mode)
    || !Number.isSafeInteger(timeoutMs)
    || timeoutMs < 1
    || timeoutMs > 60_000
  ) throw new Error(ERROR);
  if (evidencePath !== undefined && !isAbsolute(evidencePath)) throw new Error(ERROR);
  if (evidencePath !== undefined) await rm(evidencePath, { force: true });
  const executable = await realpath(binary);
  const started = Date.now();
  return new Promise((resolveProbe, rejectProbe) => {
    const child = spawnProcess(executable, [`--voice-probe-exit=${mode}`], {
      cwd,
      env: evidencePath === undefined
        ? process.env
        : { ...process.env, VOICE_PROBE_EXIT_EVIDENCE: evidencePath },
      shell: false,
      stdio: "ignore",
      windowsHide: true,
    });
    const pid = child.pid;
    if (!Number.isSafeInteger(pid) || pid < 1) {
      child.kill("SIGTERM");
      rejectProbe(new Error(ERROR));
      return;
    }
    const timeout = setTimeout(() => child.kill("SIGTERM"), timeoutMs);
    child.once("error", (error) => {
      clearTimeout(timeout);
      rejectProbe(error);
    });
    child.once("exit", async (code, signal) => {
      clearTimeout(timeout);
      const elapsedMs = Date.now() - started;
      const exited = signal === null && code === 0 && elapsedMs >= 400;
      const cleanupConfirmed = mode === "cooperative"
        && evidencePath !== undefined
        && await readFile(evidencePath, "utf8").then(
          (value) => value === "cooperative-released\n",
          () => false,
        );
      resolveProbe({
        mode,
        pid,
        exited,
        forced: mode === "forced",
        cleanupConfirmed: exited && cleanupConfirmed,
        elapsedMs,
        code,
        signal,
      });
    });
  });
}

export async function writeClosureReport(path, reports) {
  if (!isAbsolute(path) || !Array.isArray(reports) || reports.length !== 2) throw new Error(ERROR);
  const parent = await realpath(dirname(path));
  const name = basename(path);
  if (!/^[A-Za-z0-9_.-]{1,128}$/u.test(name)) throw new Error(ERROR);
  const target = resolve(parent, name);
  const temporary = `${target}.tmp-${process.pid}`;
  await writeFile(temporary, `${JSON.stringify({ version: 1, reports }, null, 2)}\n`, {
    encoding: "utf8",
    mode: 0o600,
    flag: "wx",
  });
  await rename(temporary, target);
}

async function runMatrix(repoRoot) {
  for (const name of ["VOICE_PROTOTYPE_DATA_DIR", "VOICE_TEST_CORPUS_DIR", "VOICE_PROTOTYPE_REPORT"]) {
    if (!isAbsolute(process.env[name] ?? "")) throw new Error(ERROR);
  }
  return runCommand("cargo", [
    "test", "--manifest-path", "src-tauri/Cargo.toml", "--lib",
    "services::voice::prototype_tests::native_model_matrix", "--", "--ignored", "--nocapture",
  ], { cwd: repoRoot, env: process.env, timeoutMs: MATRIX_TIMEOUT_MS });
}

async function main() {
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  const [command, ...args] = process.argv.slice(2);
  if (command === "matrix" && args.length === 0) process.exitCode = await runMatrix(repoRoot);
  else if (command === "exit" && args.length === 2) {
    const reports = [];
    const evidencePath = resolve(dirname(args[1]), "voice-probe-cooperative.marker");
    reports.push(await observeExit(args[0], "cooperative", { cwd: repoRoot, evidencePath }));
    reports.push(await observeExit(args[0], "forced", { cwd: repoRoot }));
    await writeClosureReport(args[1], reports);
    process.exitCode = reports.every((report) => report.exited) ? 0 : 1;
  } else throw new Error(ERROR);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  });
}
