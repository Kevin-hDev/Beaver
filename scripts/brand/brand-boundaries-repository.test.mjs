import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, unlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  MAX_GIT_ENTRIES,
  MAX_SCANNED_FILES,
  loadTrackedEntries,
  parseTrackedFiles,
  selectScannableFiles,
} from "./brand-boundaries-repository.mjs";

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

test("les ressources binaires ne consomment pas le budget des textes", () => {
  const files = Array.from(
    { length: MAX_SCANNED_FILES + 1 },
    (_, index) => `icons/icon-${index}.png`,
  );
  files.push("src/app.ts");

  assert.deepEqual(selectScannableFiles(files, new Set()), ["src/app.ts"]);
});

test("refuse un inventaire Git ou texte au-delà de sa borne", () => {
  const raw = `${Array.from(
    { length: MAX_GIT_ENTRIES + 1 },
    (_, index) => `asset-${index}.png`,
  ).join("\0")}\0`;
  assert.throws(() => parseTrackedFiles(raw), /trop d'entrées Git/u);

  const textFiles = Array.from(
    { length: MAX_SCANNED_FILES + 1 },
    (_, index) => `src/file-${index}.ts`,
  );
  assert.throws(
    () => selectScannableFiles(textFiles, new Set()),
    /trop de fichiers texte/u,
  );
});
