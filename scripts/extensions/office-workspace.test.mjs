import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hostDirectory } from "./office-test-helpers.mjs";

test("atomic Office writes never zeroize their caller's buffer", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-atomic-office-"));
  const workspaceModule = pathToFileURL(
    join(hostDirectory, "builtin-plugins/common/workspace.mjs"),
  );
  const { atomicWrite } = await import(workspaceModule.href);
  const source = Buffer.from("caller-owned-data");
  try {
    await atomicWrite(join(workspace, "output.bin"), source);

    assert.equal(source.toString(), "caller-owned-data");
    assert.equal(
      (await readFile(join(workspace, "output.bin"))).toString(),
      "caller-owned-data",
    );
  } finally {
    source.fill(0);
    await rm(workspace, { recursive: true, force: true });
  }
});
