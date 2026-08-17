import { spawnSync } from "node:child_process";
import { realpathSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const FILTER_PATTERN = /^[A-Za-z0-9_:]{1,256}$/u;
const FEATURES_PATTERN = /^[A-Za-z0-9_,.-]{1,128}$/u;
const MAX_TEST_THREADS = 64;
const MAX_FAILURE_CHANNEL_CHARS = 8 * 1024;
const FAILURE_MESSAGE = "Rust test filter failed";

function rejectInvalid(condition) {
  if (condition) throw new Error(FAILURE_MESSAGE);
}

function boundedChannel(value) {
  const text = String(value);
  if (text.length <= MAX_FAILURE_CHANNEL_CHARS) return text;
  const half = Math.floor(MAX_FAILURE_CHANNEL_CHARS / 2);
  return `${text.slice(0, half)}\n... diagnostic truncated ...\n${text.slice(-half)}`;
}

function inventoryFailure(result) {
  const status = Number.isInteger(result.status) ? result.status : "none";
  const signal = result.signal || "none";
  const details = [
    result.error ? `error: ${boundedChannel(result.error.message || result.error)}` : "",
    result.stdout ? `stdout:\n${boundedChannel(result.stdout)}` : "",
    result.stderr ? `stderr:\n${boundedChannel(result.stderr)}` : "",
  ]
    .filter(Boolean)
    .join("\n");
  const suffix = details ? `\n${details}` : "";
  return new Error(
    `${FAILURE_MESSAGE} during inventory (status=${status}, signal=${signal}).${suffix}`,
  );
}

export function buildCargoCommands({
  filter,
  features,
  exact = false,
  ignored = false,
  nocapture = false,
  testThreads = 1,
}) {
  rejectInvalid(typeof filter !== "string" || !FILTER_PATTERN.test(filter));
  rejectInvalid(
    features !== undefined
      && (typeof features !== "string" || !FEATURES_PATTERN.test(features)),
  );
  rejectInvalid(
    !Number.isInteger(testThreads)
      || testThreads < 1
      || testThreads > MAX_TEST_THREADS,
  );

  const cargoArgs = ["test", "--lib"];
  if (features) cargoArgs.push("--features", features);
  cargoArgs.push(filter, "--", `--test-threads=${testThreads}`);
  if (ignored) cargoArgs.push("--ignored");
  if (exact) cargoArgs.push("--exact");

  const inventory = [...cargoArgs, "--list"];
  const execute = [...cargoArgs];
  if (nocapture) execute.push("--nocapture");
  return { inventory, execute };
}

export function countListedTests(output) {
  return String(output)
    .split(/\r?\n/u)
    .filter((line) => line.trimEnd().endsWith(": test")).length;
}

export function runFilteredRustTests(config, run = spawnSync) {
  const commands = buildCargoCommands(config);
  const inventory = run("cargo", commands.inventory, {
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
    shell: false,
  });
  if (inventory.error || inventory.signal || inventory.status !== 0) {
    throw inventoryFailure(inventory);
  }

  const count = countListedTests(inventory.stdout);
  rejectInvalid(count === 0);

  const execution = run("cargo", commands.execute, {
    shell: false,
    stdio: "inherit",
  });
  rejectInvalid(execution.error || execution.signal || execution.status !== 0);
  return count;
}

function parseArguments(argv) {
  const config = {};
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--exact" || argument === "--ignored" || argument === "--nocapture") {
      config[argument.slice(2)] = true;
      continue;
    }
    const value = argv[index + 1];
    rejectInvalid(value === undefined);
    if (argument === "--filter") config.filter = value;
    else if (argument === "--features") config.features = value;
    else if (argument === "--test-threads") config.testThreads = Number(value);
    else rejectInvalid(true);
    index += 1;
  }
  return config;
}

function canonicalPath(path) {
  let canonical;
  try {
    canonical = realpathSync.native(resolve(path));
  } catch {
    canonical = resolve(path);
  }
  return process.platform === "win32" ? canonical.toLocaleLowerCase("en-US") : canonical;
}

export function isDirectExecution(moduleUrl, argvPath) {
  return Boolean(argvPath)
    && canonicalPath(fileURLToPath(moduleUrl)) === canonicalPath(argvPath);
}

if (isDirectExecution(import.meta.url, process.argv[1])) {
  try {
    const count = runFilteredRustTests(parseArguments(process.argv.slice(2)));
    process.stdout.write(`Verified ${count} filtered Rust test(s).\n`);
  } catch (error) {
    const message = error instanceof Error && error.message.startsWith(FAILURE_MESSAGE)
      ? error.message
      : FAILURE_MESSAGE;
    process.stderr.write(`${message}\n`);
    process.exitCode = 1;
  }
}
