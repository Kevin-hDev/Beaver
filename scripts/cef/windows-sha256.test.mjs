import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import test from "node:test";

test("Windows CEF hashing does not depend on the ambient Get-FileHash command", async (t) => {
  if (process.platform !== "win32") {
    t.skip("Windows PowerShell is required");
    return;
  }

  const fixtureDirectory = await mkdtemp(join(tmpdir(), "beaver-sha256-"));
  t.after(() => rm(fixtureDirectory, { recursive: true, force: true }));
  const fixturePath = join(fixtureDirectory, "fixture.bin");
  await writeFile(fixturePath, "abc");

  const command = [
    "$ErrorActionPreference = 'Stop'",
    ". $env:BEAVER_HASH_HELPER",
    "function Get-FileHash { throw 'ambient hash command used' }",
    "$digest = Get-BeaverFileSha256 -Path $env:BEAVER_HASH_FIXTURE",
    "[Console]::Out.Write($digest)",
  ].join("; ");
  const result = spawnSync(
    "powershell.exe",
    ["-NoProfile", "-NonInteractive", "-Command", command],
    {
      encoding: "utf8",
      env: {
        ...process.env,
        BEAVER_HASH_FIXTURE: fixturePath,
        BEAVER_HASH_HELPER: resolve("src-tauri/scripts/file-sha256.ps1"),
      },
      shell: false,
    },
  );

  assert.equal(result.status, 0, result.stderr);
  assert.equal(
    result.stdout,
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
});
