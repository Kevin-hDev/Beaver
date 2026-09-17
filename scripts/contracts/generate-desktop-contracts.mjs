import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const tests = [
  "services::browser::test_modules::browser_contract_tests::export_browser_contract",
  "services::browser::test_modules::favicon_contract_tests::export_favicon_contract",
  "services::terminal::terminal_contract_tests::export_terminal_contract",
];

for (const test of tests) {
  const result = spawnSync("cargo", ["test", "--lib", test, "--", "--ignored", "--exact"], {
    cwd: fileURLToPath(new URL("../../src-tauri", import.meta.url)),
    shell: false,
    stdio: "inherit",
    windowsHide: true,
  });
  if (result.status !== 0) {
    process.exitCode = result.status ?? 1;
    break;
  }
}
