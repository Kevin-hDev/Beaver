import { resolve } from "node:path";
import { buildFrontend } from "../build/frontend-build.mjs";
import { runCommand } from "../build/command-runner.mjs";
import { canonicalE2eRepoRoot } from "./e2e-process.mjs";

const repoRoot = await canonicalE2eRepoRoot(import.meta.url);

try {
  await runCommand({
    command: process.execPath,
    args: [resolve(repoRoot, "scripts/extensions/prepare-extension-host.mjs")],
    cwd: repoRoot,
  });
  await runCommand({
    command: process.execPath,
    args: [resolve(repoRoot, "scripts/cef/prepare-cef-source.mjs")],
    cwd: repoRoot,
  });
  await buildFrontend({ repoRoot });
  if (process.platform === "darwin") {
    await runCommand({
      command: "bash",
      args: [resolve(repoRoot, "src-tauri/scripts/prepare-cef.sh")],
      cwd: resolve(repoRoot, "src-tauri"),
    });
  }
} catch {
  process.stderr.write("E2E build preparation failed\n");
  process.exitCode = 1;
}
