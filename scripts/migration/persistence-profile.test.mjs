import assert from "node:assert/strict";
import { existsSync, readFileSync, statSync } from "node:fs";
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
const MAX_DOMAINS = 32;
const MAX_PROFILE_PATHS = 96;
const MAX_CONTRACTS = 256;
const MAX_SNIPPETS = 512;
const EXPECTED_DOMAINS = Object.freeze([
  "vault-api",
  "llm-oauth",
  "mcp-oauth",
  "agent-sessions",
  "tabs-permissions",
  "global-memory",
  "project-memory",
  "external-agents",
  "git-projects",
  "subagents",
  "ollama",
  "forecast-v2",
  "forecast-models",
  "forecast-runtime",
  "forecast-profiles-workbench",
  "browser",
  "local-sites",
  "providers",
  "gateway",
  "scheduler",
  "terminal",
  "mascot",
  "theme-language",
  "autostart",
  "single-instance",
]);

function safeRelativePath(value) {
  assert.equal(typeof value, "string");
  assert.ok(value.length > 0 && value.length <= 256);
  assert.equal(isAbsolute(value), false);
  assert.equal(value.includes("\0"), false);
  assert.doesNotMatch(value, /[:\r\n]/u);
  assert.equal(value.split(/[\\/]/u).includes(".."), false);
  const absolute = resolve(ROOT, value);
  assert.ok(relative(ROOT, absolute) && !relative(ROOT, absolute).startsWith(".."));
  return absolute;
}

function boundedRead(path, maxBytes) {
  const size = statSync(path).size;
  assert.ok(size <= maxBytes, `${path} dépasse la taille autorisée`);
  return readFileSync(path, "utf8");
}

function loadManifest() {
  return JSON.parse(boundedRead(MANIFEST_PATH, MAX_MANIFEST_BYTES));
}

test("le profil couvre exactement les 25 domaines du plan", () => {
  const manifest = loadManifest();
  assert.equal(manifest.schemaVersion, 1);
  assert.ok(manifest.domains.length <= MAX_DOMAINS);
  assert.deepEqual(
    manifest.domains.map(({ id }) => id),
    EXPECTED_DOMAINS,
  );
  let contractCount = 0;
  let snippetCount = 0;
  for (const domain of manifest.domains) {
    assert.ok(domain.contracts.length > 0, `${domain.id}: aucun contrat`);
    assert.ok(domain.evidence.length > 0, `${domain.id}: aucune preuve`);
    assert.ok(domain.manualGates.length <= 8, `${domain.id}: trop de portes`);
    contractCount += domain.contracts.length;
    for (const contract of domain.contracts) {
      assert.ok(["baseline", "migration"].includes(contract.scope ?? "baseline"));
      snippetCount += contract.snippets.length;
    }
    assert.ok(contractCount <= MAX_CONTRACTS);
    assert.ok(snippetCount <= MAX_SNIPPETS);
  }
});

test("les identités historiques restent inchangées", () => {
  const { baseline } = loadManifest();
  assert.deepEqual(baseline, {
    version: "1.0.2",
    commit: "fdf43447ac8444683527cb06f9f5669407d7c12f",
    dataRoot: ".local/share/cl-go-dash",
    bundleIdentifier: "com.clgo.dash",
    keyringService: "cl-go-dash",
    keyringUser: "master-key",
  });
});

test("chaque chemin de profil est borné, relatif et conserve le nom CL-GO", () => {
  const paths = loadManifest().domains.flatMap(({ profilePaths }) => profilePaths);
  assert.ok(paths.length > 0 && paths.length <= MAX_PROFILE_PATHS);
  for (const path of paths) {
    safeRelativePath(path);
    assert.doesNotMatch(path, /(^|[\\/])beaver([\\/]|$)/iu);
  }
  assert.equal(new Set(paths).size < paths.length, true, "config.json doit être partagé");
});

test("tous les contrats courants existent réellement dans les sources", () => {
  const manifest = loadManifest();
  for (const domain of manifest.domains) {
    for (const contract of domain.contracts) {
      const path = safeRelativePath(contract.file);
      if ((contract.scope ?? "baseline") === "baseline" && !existsSync(path)) continue;
      const source = boundedRead(path, MAX_SOURCE_BYTES);
      assert.ok(contract.snippets.length > 0 && contract.snippets.length <= 16);
      for (const snippet of contract.snippets) {
        assert.ok(
          source.includes(snippet),
          `${domain.id}: contrat absent dans ${contract.file}: ${snippet}`,
        );
      }
    }
    for (const evidence of domain.evidence) {
      assert.ok(statSync(safeRelativePath(evidence)).isFile(), `${domain.id}: preuve absente`);
    }
  }
});

test("aucune nouvelle identité de stockage Beaver n’est introduite", () => {
  const manifest = loadManifest();
  const forbidden = [
    ".local/share/beaver",
    "com.beaver.",
    "beaver-theme",
    "beaver-language",
    "beaver-session-key",
  ];
  const sourceFiles = new Set(
    manifest.domains.flatMap(({ contracts }) => contracts.map(({ file }) => file))
      .filter((file) => existsSync(safeRelativePath(file))),
  );
  for (const file of sourceFiles) {
    const source = boundedRead(safeRelativePath(file), MAX_SOURCE_BYTES).toLowerCase();
    for (const value of forbidden) {
      assert.equal(source.includes(value), false, `${file}: identité interdite ${value}`);
    }
  }
});

test("une migration incomplète bloque le démarrage sans exposer de chemin", () => {
  const startup = boundedRead(
    safeRelativePath("src-tauri/src/lib.rs"),
    MAX_SOURCE_BYTES,
  );
  assert.match(
    startup,
    /storage_migration::initialize\(app\.handle\(\)\)\.map_err\(std::io::Error::other\)\?/u,
  );
  const migration = boundedRead(
    safeRelativePath("src-tauri/src/storage_migration.rs"),
    MAX_SOURCE_BYTES,
  );
  assert.match(migration, /run\(app_handle\)\?;/u);
  assert.match(migration, /private_store::repair_app_storage\(\)/u);
  assert.doesNotMatch(startup, /if let Err\(e\) = storage_migration::/u);
  for (const file of [
    "src-tauri/src/storage_migration.rs",
    "src-tauri/src/storage_migration_files.rs",
    "src-tauri/src/services/agent_local/subagent_startup_cleanup.rs",
    "src/hooks/terminal-persistence.ts",
  ]) {
    const source = boundedRead(safeRelativePath(file), MAX_SOURCE_BYTES);
    assert.doesNotMatch(source, /eprintln![\s\S]{0,160}\.display\(\)/u);
    assert.doesNotMatch(source, /console\.warn\([^\n]*,\s*(?:err|error)\b/u);
  }
});
