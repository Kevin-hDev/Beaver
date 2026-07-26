import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync, statSync } from "node:fs";
import { isAbsolute, relative, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("../../", import.meta.url));
const MANIFEST_PATH = resolve(
  ROOT,
  "scripts/migration/cl-go-v1.0.2-profile.json",
);
const MAX_MANIFEST_BYTES = 128 * 1024;
const MAX_SOURCE_BYTES = 2 * 1024 * 1024;
const MAX_CONTRACTS = 256;
const MAX_SNIPPETS = 512;

function loadManifest() {
  assert.ok(statSync(MANIFEST_PATH).size <= MAX_MANIFEST_BYTES);
  return JSON.parse(readFileSync(MANIFEST_PATH, "utf8"));
}

function safeRelativePath(value) {
  assert.equal(typeof value, "string");
  assert.ok(value.length > 0 && value.length <= 256);
  assert.equal(isAbsolute(value), false);
  assert.doesNotMatch(value, /[:\0\r\n]/u);
  assert.equal(value.split(/[\\/]/u).includes(".."), false);
  const absolute = resolve(ROOT, value);
  const inside = relative(ROOT, absolute);
  assert.ok(inside && !inside.startsWith(".."));
}

function readBaselineFile(commit, file) {
  assert.match(commit, /^[a-f0-9]{40}$/u);
  safeRelativePath(file);
  return execFileSync("git", ["show", `${commit}:${file}`], {
    cwd: ROOT,
    encoding: "utf8",
    maxBuffer: MAX_SOURCE_BYTES,
    stdio: ["ignore", "pipe", "pipe"],
  });
}

test("les contrats historiques existaient dans le commit de référence", () => {
  const manifest = loadManifest();
  execFileSync("git", ["merge-base", "--is-ancestor", manifest.baseline.commit, "HEAD"], {
    cwd: ROOT,
    maxBuffer: 1024,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const packageJson = JSON.parse(
    readBaselineFile(manifest.baseline.commit, "package.json"),
  );
  assert.equal(packageJson.version, manifest.baseline.version);
  let contractCount = 0;
  let snippetCount = 0;
  for (const domain of manifest.domains) {
    for (const contract of domain.contracts) {
      contractCount += 1;
      assert.ok(contractCount <= MAX_CONTRACTS);
      if (contract.scope === "migration") continue;
      const source = readBaselineFile(manifest.baseline.commit, contract.file);
      for (const snippet of contract.snippets) {
        snippetCount += 1;
        assert.ok(snippetCount <= MAX_SNIPPETS);
        assert.ok(
          source.includes(snippet),
          `${domain.id}: contrat historique absent dans ${contract.file}`,
        );
      }
    }
  }
  assert.ok(snippetCount > 0);
});
