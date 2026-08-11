import { readFile } from "node:fs/promises";

import { isDirectExecution } from "./direct-execution.mjs";

export { isDirectExecution } from "./direct-execution.mjs";

const FORBIDDEN_SANDBOX_BYPASS = /(?:no[_-]sandbox|CEF_NO_SANDBOX)/iu;

export function validateCefSupervisionContracts({ workflow, build, macHelper }) {
  const errors = [];
  requireText(workflow, "backend-windows-native:", "Windows native job is missing", errors);
  requireText(workflow, "backend-macos-native:", "macOS native job is missing", errors);
  requireText(
    workflow,
    "services::browser::cef_supervision::windows_tracker_tests",
    "Windows supervision tests are missing",
    errors,
  );
  requireText(
    workflow,
    "services::browser::cef_supervision::macos_tracker_tests",
    "macOS supervision tests are missing",
    errors,
  );
  if (FORBIDDEN_SANDBOX_BYPASS.test(workflow)) {
    errors.push("CEF sandbox bypass is forbidden");
  }
  if (/target\s*==\s*"linux"[\s\S]{0,160}native_browser/u.test(build)) {
    errors.push("Linux native_browser must remain disabled");
  }
  const sandbox = macHelper.indexOf("sandbox.initialize");
  const admission = macHelper.indexOf("admit_after_sandbox");
  if (sandbox < 0 || admission < 0 || sandbox > admission) {
    errors.push("macOS helper admission must follow sandbox initialization");
  }
  return errors;
}

function requireText(source, expected, message, errors) {
  if (!source.includes(expected)) errors.push(message);
}

export async function validateRepository() {
  const root = new URL("../../", import.meta.url);
  const [workflow, build, macHelper] = await Promise.all([
    readFile(new URL(".github/workflows/ci.yml", root), "utf8"),
    readFile(new URL("src-tauri/build.rs", root), "utf8"),
    readFile(new URL("src-tauri/src/services/browser/macos_helper_entry.rs", root), "utf8"),
  ]);
  return validateCefSupervisionContracts({ workflow, build, macHelper });
}

if (isDirectExecution(import.meta.url, process.argv[1])) {
  const errors = await validateRepository();
  if (errors.length > 0) {
    process.stderr.write(`${errors.join("\n")}\n`);
    process.exitCode = 1;
  }
}
