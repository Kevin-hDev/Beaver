import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const result = spawnSync(
  "cargo",
  ["test", "--lib", "models::voice_contract_tests::export_typescript_voice_contract", "--", "--ignored", "--exact", "--nocapture"],
  {
    cwd: fileURLToPath(new URL("../../src-tauri", import.meta.url)),
    shell: false,
    stdio: "inherit",
    windowsHide: true,
  },
);
process.exitCode = result.status ?? 1;
