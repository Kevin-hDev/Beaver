import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, unlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { loadTrackedEntries } from "./brand-boundaries-repository.mjs";

function git(root, args) {
  execFileSync("git", ["-C", root, ...args], {
    encoding: "utf8",
    shell: false,
    stdio: "pipe",
  });
}

test("analyse aussi les nouveaux fichiers non ignorés", (context) => {
  const root = mkdtempSync(join(tmpdir(), "beaver-brand-boundaries-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  git(root, ["init", "--quiet"]);

  writeFileSync(join(root, ".gitignore"), "ignored.ts\n", "utf8");
  writeFileSync(join(root, "tracked.ts"), 'export const tracked = "ok";\n', "utf8");
  writeFileSync(join(root, "untracked.ts"), 'export const untracked = "ok";\n', "utf8");
  writeFileSync(join(root, "installer.nsh"), '!define PRODUCT "ok"\n', "utf8");
  writeFileSync(join(root, "ignored.ts"), 'export const ignored = "no";\n', "utf8");
  git(root, ["add", ".gitignore", "tracked.ts"]);

  const files = loadTrackedEntries(root).map((entry) => entry.file).sort();

  assert.deepEqual(files, ["installer.nsh", "tracked.ts", "untracked.ts"]);
});

test("ignore un fichier suivi qui vient d'être supprimé", (context) => {
  const root = mkdtempSync(join(tmpdir(), "beaver-brand-deletion-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  git(root, ["init", "--quiet"]);

  writeFileSync(join(root, "kept.ts"), 'export const kept = "ok";\n', "utf8");
  writeFileSync(join(root, "deleted.ts"), 'export const removed = "ok";\n', "utf8");
  git(root, ["add", "kept.ts", "deleted.ts"]);
  unlinkSync(join(root, "deleted.ts"));

  const files = loadTrackedEntries(root).map((entry) => entry.file);

  assert.deepEqual(files, ["kept.ts"]);
});
