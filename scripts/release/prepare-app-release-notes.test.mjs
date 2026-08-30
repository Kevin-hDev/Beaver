import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import test from "node:test";

const SCRIPT = join(
  dirname(fileURLToPath(import.meta.url)),
  "prepare-app-release-notes.mjs",
);
const REPOSITORY_ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

function localizedNotes(label) {
  return {
    fr: [`${label} en francais.`],
    en: [`${label} in English.`],
    es: [`${label} en espanol.`],
    de: [`${label} auf Deutsch.`],
    it: [`${label} in italiano.`],
    zh: [`${label} 中文。`],
    ja: [`${label} 日本語。`],
  };
}

function run(directory, ...arguments_) {
  return spawnSync(process.execPath, [SCRIPT, ...arguments_], {
    cwd: directory,
    encoding: "utf8",
  });
}

test("archives every older release while keeping the published payload bounded", () => {
  const directory = mkdtempSync(join(tmpdir(), "beaver-release-notes-"));
  const current = localizedNotes("Current release");
  const previous = localizedNotes("Previous release");
  const archived = localizedNotes("Archived release");
  writeFileSync(
    join(directory, "app-release-notes.json"),
    `${JSON.stringify({ "1.1.9": current, "1.1.8": previous }, null, 2)}\n`,
  );
  writeFileSync(
    join(directory, "app-release-notes-archive.json"),
    `${JSON.stringify({ "1.1.7": archived }, null, 2)}\n`,
  );

  try {
    const prepared = run(directory, "1.1.9");
    assert.equal(prepared.status, 0, prepared.stderr);

    const activePath = join(directory, "app-release-notes.json");
    const archivePath = join(directory, "app-release-notes-archive.json");
    assert.deepEqual(JSON.parse(readFileSync(activePath, "utf8")), {
      "1.1.9": current,
    });
    assert.deepEqual(JSON.parse(readFileSync(archivePath, "utf8")), {
      "1.1.8": previous,
      "1.1.7": archived,
    });
    assert.ok(Buffer.byteLength(readFileSync(activePath, "utf8")) <= 64 * 1024);

    const firstActive = readFileSync(activePath, "utf8");
    const firstArchive = readFileSync(archivePath, "utf8");
    const repeated = run(directory, "v1.1.9");
    assert.equal(repeated.status, 0, repeated.stderr);
    assert.equal(readFileSync(activePath, "utf8"), firstActive);
    assert.equal(readFileSync(archivePath, "utf8"), firstArchive);

    const checked = run(directory, "1.1.9", "--check");
    assert.equal(checked.status, 0, checked.stderr);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("check mode rejects an unprepared payload without modifying either file", () => {
  const directory = mkdtempSync(join(tmpdir(), "beaver-release-notes-"));
  const activePath = join(directory, "app-release-notes.json");
  const archivePath = join(directory, "app-release-notes-archive.json");
  const active = `${JSON.stringify({
    "1.1.9": localizedNotes("Current release"),
    "1.1.8": localizedNotes("Previous release"),
  }, null, 2)}\n`;
  const archive = `${JSON.stringify({}, null, 2)}\n`;
  writeFileSync(activePath, active);
  writeFileSync(archivePath, archive);

  try {
    const checked = run(directory, "1.1.9", "--check");
    assert.notEqual(checked.status, 0);
    assert.match(checked.stderr, /not prepared/u);
    assert.equal(readFileSync(activePath, "utf8"), active);
    assert.equal(readFileSync(archivePath, "utf8"), archive);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("the tagged repository payload is already prepared before release builds", () => {
  const checked = run(REPOSITORY_ROOT, "1.1.9", "--check");
  assert.equal(checked.status, 0, checked.stderr);
});

test("rejects an invalid version without exposing a stack trace", () => {
  const rejected = run(REPOSITORY_ROOT, "../1.1.9");
  assert.notEqual(rejected.status, 0);
  assert.equal(rejected.stderr, "A stable release version is required.\n");
});
