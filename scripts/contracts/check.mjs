import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { CONTRACT_TESTS } from "./test-list.mjs";

const MAX_OUTPUT_BYTES = 16 * 1024 * 1024;

export function assertExactSuccess(name, result) {
  if (result.error || result.status !== 0
    || !result.stdout?.split(/\r?\n/).includes(`test ${name} ... ok`)
    || !/^test result: ok\. 1 passed; 0 failed; 0 ignored;/m.test(result.stdout)) {
    throw new Error(`Contract test was not collected and passed exactly once: ${name}`);
  }
}

export function runExactTest(name) {
  const result = spawnSync("cargo", ["test", "--lib", name, "--", "--exact"], {
    cwd: fileURLToPath(new URL("../../src-tauri", import.meta.url)),
    encoding: "utf8", maxBuffer: MAX_OUTPUT_BYTES,
  });
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  assertExactSuccess(name, result);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  try {
    for (const name of CONTRACT_TESTS) runExactTest(name);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
