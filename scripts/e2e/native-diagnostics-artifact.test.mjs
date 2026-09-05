import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

test("the persisted native report contains only bounded safe categories", async () => {
  const artifact = await import("./native-diagnostics-artifact.mjs").catch(() => ({}));
  assert.equal(typeof artifact.persistNativeDiagnostics, "function");
  if (typeof artifact.persistNativeDiagnostics !== "function") return;
  const root = await mkdtemp(join(tmpdir(), "beaver-native-artifact-"));
  const logs = join(root, "logs");
  const output = join(root, "artifacts");
  await mkdir(logs);
  try {
    await writeFile(
      join(logs, "wdio-safe.log"),
      "token=secret\n[e2e-lifecycle] setup-completed\n",
      "utf8",
    );
    await artifact.persistNativeDiagnostics(logs, output);
    const report = await readFile(join(output, "native-diagnostics.txt"), "utf8");
    assert.equal(report, "application-stage:setup-completed\n");
    assert.equal(report.includes("secret"), false);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("a pre-created temporary symlink cannot redirect the diagnostic report", async () => {
  const { persistNativeDiagnostics } = await import("./native-diagnostics-artifact.mjs");
  const root = await mkdtemp(join(tmpdir(), "beaver-native-artifact-"));
  const logs = join(root, "logs");
  const output = join(root, "artifacts");
  const protectedFile = join(root, "protected.txt");
  await mkdir(logs);
  await mkdir(output);
  try {
    await writeFile(
      join(logs, "wdio-safe.log"),
      "[e2e-lifecycle] setup-completed\n",
      "utf8",
    );
    await writeFile(protectedFile, "unchanged\n", "utf8");
    await symlink(protectedFile, join(output, ".native-diagnostics.tmp"));

    await persistNativeDiagnostics(logs, output);

    assert.equal(await readFile(protectedFile, "utf8"), "unchanged\n");
    assert.equal(
      await readFile(join(output, "native-diagnostics.txt"), "utf8"),
      "application-stage:setup-completed\n",
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
