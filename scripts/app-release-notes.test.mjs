import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";

const EXPECTED_HEADING = "### App release notes";
const CURRENT_VERSION = JSON.parse(readFileSync("package.json", "utf8")).version;

function runNotes(arguments_, environment = {}) {
  return spawnSync(
    process.execPath,
    ["scripts/app-release-notes.mjs", ...arguments_],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      env: { ...process.env, ...environment },
    },
  );
}

test("writes a bounded GitHub output by default", () => {
  const directory = mkdtempSync(join(tmpdir(), "beaver-release-notes-"));
  const output = join(directory, "github-output.txt");
  writeFileSync(output, "", "utf8");

  try {
    const result = runNotes([CURRENT_VERSION], { GITHUB_OUTPUT: output });

    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, "");
    assert.match(readFileSync(output, "utf8"), /body<<EOF\n### App release notes/u);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("stdout mode remains explicit inside GitHub Actions", () => {
  const directory = mkdtempSync(join(tmpdir(), "beaver-release-notes-"));
  const output = join(directory, "github-output.txt");
  writeFileSync(output, "", "utf8");

  try {
    const result = runNotes([CURRENT_VERSION, "--stdout"], { GITHUB_OUTPUT: output });

    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, new RegExp(`^${EXPECTED_HEADING}`, "u"));
    assert.equal(readFileSync(output, "utf8"), "");
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

test("rejects unknown output modes", () => {
  const result = runNotes(["1.1.5", "--unknown"]);

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Invalid release notes arguments/u);
});
