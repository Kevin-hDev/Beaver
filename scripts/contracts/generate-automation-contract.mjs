import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const result = spawnSync("cargo", [
  "test", "--lib",
  "services::automations::contract_export::export_typescript_automation_contract",
  "--", "--ignored", "--exact", "--nocapture",
], {
  cwd: fileURLToPath(new URL("../../src-tauri", import.meta.url)),
  shell: false,
  stdio: "inherit",
  windowsHide: true,
});
process.exitCode = result.status ?? 1;
