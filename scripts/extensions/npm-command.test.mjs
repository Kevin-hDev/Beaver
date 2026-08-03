import assert from "node:assert/strict";
import { mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

test("npm preparation executes the npm CLI through Node", async () => {
  const temporary = await mkdtemp(join(tmpdir(), "beaver-npm-command-"));
  const npmCli = join(temporary, "npm-cli.js");
  await writeFile(npmCli, "// npm fixture\n", { mode: 0o600 });

  try {
    const { createNpmInvocation } = await import("./npm-command.mjs");
    const invocation = await createNpmInvocation(
      ["ci", "--ignore-scripts"],
      npmCli,
    );

    assert.equal(invocation.program, process.execPath);
    assert.deepEqual(invocation.args, [
      await realpath(npmCli),
      "ci",
      "--ignore-scripts",
    ]);
  } finally {
    await rm(temporary, { recursive: true, force: true });
  }
});

test("npm preparation rejects invalid command inputs", async () => {
  const { createNpmInvocation } = await import("./npm-command.mjs");

  await assert.rejects(
    createNpmInvocation([], "relative/npm-cli.js"),
    /Invalid npm runtime/,
  );
  await assert.rejects(
    createNpmInvocation(Array(17).fill("value"), process.execPath),
    /Invalid npm runtime/,
  );
});
