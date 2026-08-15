import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, mkdir, writeFile } from "node:fs/promises";
import { join, dirname } from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { verifyOllamaManagerContracts } from "./ollama-manager-contracts.mjs";

const FIXTURE_ROOT = join(dirname(new URL(import.meta.url).pathname), "fixtures/contracts");

async function readFixture(name) {
  return readFile(join(FIXTURE_ROOT, name), "utf8");
}

async function fixtureRepository(name, relativePath = "src-tauri/src/services/agent_local/fixture.rs") {
  const root = await mkdtemp(join(tmpdir(), "ollama-manager-contracts-"));
  const path = join(root, relativePath);
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, await readFixture(name), "utf8");
  return { root, path };
}

async function verifyFixture(name) {
  const fixture = await fixtureRepository(name);
  try {
    return await verifyOllamaManagerContracts({ repoRoot: fixture.root });
  } finally {
    await rm(fixture.root, { recursive: true, force: true });
  }
}

test("autorise un appel via OllamaManager hors du domaine", async () => {
  const result = await verifyFixture("allowed-manager-call.txt");
  assert.deepEqual(result.violations, []);
  assert.equal(result.scannedFiles, 1);
});

test("interdit les chemins transactionnels hors autorité", async () => {
  const result = await verifyFixture("forbidden-direct-path.txt");
  assert.equal(result.violations.length, 1);
  assert.equal(result.violations[0].rule, "transactional-path");
});

test("interdit le lancement direct du binaire", async () => {
  const result = await verifyFixture("forbidden-command-spawn.txt");
  assert.equal(result.violations.length, 1);
  assert.equal(result.violations[0].rule, "binary-spawn");
});

test("interdit un calendrier de reprise concurrent", async () => {
  const result = await verifyFixture("forbidden-second-retry.txt");
  assert.equal(result.violations.length, 1);
  assert.equal(result.violations[0].rule, "duplicate-retry-calendar");
});

test("ignore les fixtures négatives et les arbres générés", async () => {
  const root = await mkdtemp(join(tmpdir(), "ollama-manager-contracts-ignore-"));
  try {
    await mkdir(join(root, "scripts/ollama/fixtures"), { recursive: true });
    await mkdir(join(root, "target/nested"), { recursive: true });
    await mkdir(join(root, "node_modules/nested"), { recursive: true });
    await mkdir(join(root, "graphify-out/nested"), { recursive: true });
    const forbidden = await readFixture("forbidden-direct-path.txt");
    for (const path of [
      "scripts/ollama/fixtures/negative.rs",
      "target/nested/ignored.rs",
      "node_modules/nested/ignored.rs",
      "graphify-out/nested/ignored.rs",
    ]) {
      await writeFile(join(root, path), forbidden, "utf8");
    }
    const source = join(root, "src-tauri/src/services/agent_local/allowed.rs");
    await mkdir(dirname(source), { recursive: true });
    await writeFile(source, await readFixture("allowed-manager-call.txt"), "utf8");
    const result = await verifyOllamaManagerContracts({ repoRoot: root });
    assert.equal(result.scannedFiles, 1);
    assert.deepEqual(result.violations, []);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("échoue quand aucun fichier source attendu n'est présent", async () => {
  const root = await mkdtemp(join(tmpdir(), "ollama-manager-contracts-empty-"));
  try {
    await assert.rejects(
      () => verifyOllamaManagerContracts({ repoRoot: root }),
      /no source files/i,
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
