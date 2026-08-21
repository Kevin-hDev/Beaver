import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  parseSupportedPythonVersion,
  probePythonCandidate,
  pythonCandidates,
  readSupportedPythonVersion,
  resolvePythonCommand,
} from "./python-runtime.mjs";

const EXPECTED_VERSION = { major: 3, minor: 14, label: "3.14" };
const VERSION_FILE = "scripts/build/searxng-python-version.txt";
const MANIFEST_GOLDEN = "src-tauri/resources/searxng-sidecar/runtime-manifest.golden.json";

function git(args) {
  return execFileSync("git", args, {
    cwd: process.cwd(),
    encoding: "utf8",
    maxBuffer: 1024,
    timeout: 5000,
  });
}

test("accepte uniquement la version SearXNG majeure et mineure contrôlée", () => {
  assert.deepEqual(parseSupportedPythonVersion("3.14\n"), EXPECTED_VERSION);
  for (const invalid of ["3", "3.14.1", "3.9", "3.14 extra", "4.0", "x".repeat(33)]) {
    assert.throws(() => parseSupportedPythonVersion(invalid), /Python runtime unavailable/);
  }
});

test("conserve LF pour le fichier de version même avec core.autocrlf", () => {
  assert.equal(git(["check-attr", "eol", "--", VERSION_FILE]), `${VERSION_FILE}: eol: lf\n`);
  const filtered = git(["-c", "core.autocrlf=true", "cat-file", "--filters", `HEAD:${VERSION_FILE}`]);
  assert.equal(filtered, "3.14\n");
  assert.deepEqual(parseSupportedPythonVersion(filtered), EXPECTED_VERSION);
});

test("conserve LF pour le contrat partagé du manifeste SearXNG", () => {
  assert.equal(
    git(["check-attr", "eol", "--", MANIFEST_GOLDEN]),
    `${MANIFEST_GOLDEN}: eol: lf\n`,
  );
});

test("lit uniquement le fichier de version contrôlé du dépôt", async () => {
  const repoRoot = await mkdtemp(join(tmpdir(), "searxng-python-runtime-"));
  const versionDirectory = join(repoRoot, "scripts", "build");
  const versionFile = join(versionDirectory, "searxng-python-version.txt");
  const internalTarget = join(repoRoot, "internal-version.txt");
  const externalRoot = await mkdtemp(join(tmpdir(), "searxng-external-version-"));
  const externalTarget = join(externalRoot, "version.txt");
  try {
    await mkdir(versionDirectory, { recursive: true });
    await writeFile(versionFile, "3.14\n");
    assert.deepEqual(readSupportedPythonVersion(repoRoot), EXPECTED_VERSION);
    await rm(versionFile);
    await writeFile(internalTarget, "3.14\n");
    await symlink("../../internal-version.txt", versionFile);
    assert.throws(() => readSupportedPythonVersion(repoRoot), /Python runtime unavailable/);
    await rm(versionFile);
    await writeFile(externalTarget, "3.14\n");
    await symlink(externalTarget, versionFile);
    assert.throws(() => readSupportedPythonVersion(repoRoot), /Python runtime unavailable/);
  } finally {
    await rm(repoRoot, { recursive: true, force: true });
    await rm(externalRoot, { recursive: true, force: true });
  }
});

test("retourne les candidats Windows exacts dans l'ordre prévu", () => {
  assert.deepEqual(pythonCandidates("win32", EXPECTED_VERSION), [
    { command: "py", prefixArgs: ["-3.14"], label: "py-3.14" },
    { command: "python3.14", prefixArgs: [], label: "python3.14" },
    { command: "python3", prefixArgs: [], label: "python3" },
    { command: "python", prefixArgs: [], label: "python" },
  ]);
});

test("retourne les candidats Unix exacts sur macOS et Linux", () => {
  const expectedCandidates = [
    { command: "python3.14", prefixArgs: [], label: "python3.14" },
    { command: "python3", prefixArgs: [], label: "python3" },
    { command: "python", prefixArgs: [], label: "python" },
  ];
  for (const platform of ["darwin", "linux"]) {
    assert.deepEqual(pythonCandidates(platform, EXPECTED_VERSION), expectedCandidates, platform);
  }
});

test("refuse un Python installé mais différent de la version supportée", async () => {
  const tried = [];
  const selected = await resolvePythonCommand({
    platform: "linux",
    expectedVersion: EXPECTED_VERSION,
    probe: async (candidate, expected) => {
      tried.push(candidate.label);
      return candidate.label === "python3.14" && expected.label === "3.14";
    },
  });

  assert.equal(selected.label, "python3.14");
  assert.deepEqual(tried, ["python3.14"]);
});

test("échoue fermée après quatre candidats au maximum", async () => {
  let attempts = 0;
  await assert.rejects(
    () => resolvePythonCommand({
      platform: "win32",
      expectedVersion: EXPECTED_VERSION,
      probe: async () => {
        attempts += 1;
        return false;
      },
    }),
    /Python runtime unavailable/,
  );
  assert.equal(attempts, 4);
});

test("refuse une demande de résolution malformée sans détail interne", async () => {
  await assert.rejects(() => resolvePythonCommand(null), /Python runtime unavailable/);
});

test("le probe vérifie exactement CPython 3.14, masque la sortie et limite à cinq secondes", async () => {
  const calls = [];
  const candidate = { command: "python", prefixArgs: ["-3.14"], label: "py-3.14" };
  const accepted = await probePythonCandidate(candidate, EXPECTED_VERSION, async (spec) => {
    calls.push(spec);
  });

  assert.equal(accepted, true);
  assert.deepEqual(calls, [{
    command: "python",
    args: ["-3.14", "-c", "import sys; raise SystemExit(0 if (sys.implementation.name, sys.version_info.major, sys.version_info.minor) == ('cpython', 3, 14) else 1)"],
    cwd: process.cwd(),
    stdio: "ignore",
    timeoutMs: 5000,
  }]);
});

test("le probe masque les erreurs et refuse le candidat", async () => {
  const accepted = await probePythonCandidate(
    { command: "python", prefixArgs: [], label: "python" },
    EXPECTED_VERSION,
    async () => { throw new Error("private detail"); },
  );

  assert.equal(accepted, false);
});
