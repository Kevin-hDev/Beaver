import assert from "node:assert/strict";
import { test } from "node:test";

import {
  probePythonCandidate,
  pythonCandidates,
  resolvePythonCommand,
} from "./python-runtime.mjs";

test("retourne les candidats Windows dans l'ordre prévu", () => {
  assert.deepEqual(pythonCandidates("win32"), [
    { command: "py", prefixArgs: ["-3"] },
    { command: "python", prefixArgs: [] },
    { command: "python3", prefixArgs: [] },
    { command: "uv", prefixArgs: ["run", "python"] },
  ]);
});

test("retourne les candidats Unix dans l'ordre prévu", () => {
  assert.deepEqual(pythonCandidates("linux"), [
    { command: "python3", prefixArgs: [] },
    { command: "python", prefixArgs: [] },
    { command: "uv", prefixArgs: ["run", "python"] },
  ]);
});

test("préfère py -3 puis python sous Windows", async () => {
  const tried = [];
  const selected = await resolvePythonCommand({
    platform: "win32",
    probe: async (candidate) => {
      tried.push(candidate);
      return candidate.command === "python";
    },
  });

  assert.deepEqual(tried.map((candidate) => candidate.command), ["py", "python"]);
  assert.deepEqual(selected, { command: "python", prefixArgs: [] });
});

test("échoue fermée après quatre candidats au maximum", async () => {
  let attempts = 0;
  await assert.rejects(
    () =>
      resolvePythonCommand({
        platform: "win32",
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

test("le probe vérifie Python 3.10+, masque la sortie et limite à cinq secondes", async () => {
  const calls = [];
  const candidate = { command: "python", prefixArgs: ["-3"] };
  const accepted = await probePythonCandidate(candidate, async (spec) => {
    calls.push(spec);
  });

  assert.equal(accepted, true);
  assert.deepEqual(calls, [
    {
      command: "python",
      args: [
        "-3",
        "-c",
        "import sys; raise SystemExit(0 if sys.version_info >= (3, 10) else 1)",
      ],
      cwd: process.cwd(),
      stdio: "ignore",
      timeoutMs: 5000,
    },
  ]);
});

test("le probe masque les erreurs et refuse le candidat", async () => {
  const accepted = await probePythonCandidate(
    { command: "python", prefixArgs: [] },
    async () => {
      throw new Error("private detail");
    },
  );

  assert.equal(accepted, false);
});
