import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync, statSync } from "node:fs";
import { relative, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { EXPECTED_DATA_DIR_REFERENCES } from "./persistence-data-dir-contracts.mjs";

const ROOT = fileURLToPath(new URL("../../", import.meta.url));
const MANIFEST = JSON.parse(
  readFileSync(resolve(ROOT, "scripts/migration/cl-go-v1.0.2-profile.json"), "utf8"),
);
const MAX_TRACKED_FILES = 5_000;
const MAX_SOURCE_BYTES = 2 * 1024 * 1024;
const MAX_FINDINGS = 512;
const DATA_JOIN =
  /(?:crate::services::paths::|super::paths::|paths::)?data_dir\(\)\s*\.join\("([^"]+)"\)/gu;
const DATA_DIR_CALL =
  /(?:crate::services::paths::|super::paths::|paths::)?data_dir\(\)/gu;

function trackedRustFiles() {
  const output = execFileSync("git", ["ls-files", "-z", "src-tauri/src"], {
    cwd: ROOT,
    encoding: "utf8",
    maxBuffer: 4 * 1024 * 1024,
  });
  const files = output
    .split("\0")
    .filter((file) => file.endsWith(".rs") && !file.includes("_tests.rs"));
  assert.ok(files.length > 0 && files.length <= MAX_TRACKED_FILES);
  return files;
}

function boundedRustSource(file) {
  const absolute = resolve(ROOT, file);
  const inside = relative(ROOT, absolute);
  assert.ok(inside && !inside.startsWith(".."));
  assert.ok(statSync(absolute).size <= MAX_SOURCE_BYTES);
  // Test-only items may precede production items, so source order must not hide accesses.
  return readFileSync(absolute, "utf8");
}

test("chaque racine Rust persistante est classée ou explicitement transitoire", () => {
  const profileRoots = new Set(
    MANIFEST.domains.flatMap(({ profilePaths }) =>
      profilePaths.map((path) => path.split("/")[0]),
    ),
  );
  const allowed = new Set([...profileRoots, ...MANIFEST.transientRoots]);
  assert.ok(allowed.size <= 128);
  const findings = [];
  for (const file of trackedRustFiles()) {
    for (const match of boundedRustSource(file).matchAll(DATA_JOIN)) {
      findings.push({ file, root: match[1].split("/")[0] });
      assert.ok(findings.length <= MAX_FINDINGS);
    }
  }
  const unknown = findings.filter(({ root }) => !allowed.has(root));
  assert.deepEqual(unknown, []);
  // Les canaux d'Hôte sont recréés à chaque lancement et ne portent aucune donnée durable.
  for (const root of MANIFEST.transientRoots) {
    assert.match(
      root,
      /(?:\.pid$|^cef-supervision$|^extension-host-channels$|^reasoning-fixture-(?:reports|runtime)$|^update-health$)/u,
    );
  }
});

test("chaque accès Rust au dossier de données reste explicitement revu", () => {
  const actual = [];
  for (const file of trackedRustFiles()) {
    const count = [...boundedRustSource(file).matchAll(DATA_DIR_CALL)].length;
    if (count > 0) actual.push([file, count]);
  }
  assert.deepEqual(actual, EXPECTED_DATA_DIR_REFERENCES);
});
