import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { createHost } from "./host-test-client.mjs";
import { hostScript } from "./office-test-helpers.mjs";

test("answers the active request before a fatal host shutdown", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-fatal-host-"));
  const source = join(directory, "index.ts");
  await writeFile(
    source,
    `export default function (api: any) {
      api.registerTool({
        name: "fatal",
        description: "Trigger an asynchronous fatal error",
        parameters: { type: "object" },
        execute() {
          setTimeout(() => { throw new Error("fatal extension error"); }, 0);
          return new Promise(() => {});
        }
      });
    }`,
    { mode: 0o600 },
  );
  const host = createHost(hostScript);
  try {
    await host.request("host.sync", {
      extensions: [{
        id: "com.beaver.fatal-test",
        mainPath: source,
        manifest: { apiLevel: "stable" },
      }],
    });
    await assert.rejects(
      host.request("tool.call", {
        name: "com.beaver.fatal-test.fatal",
        arguments: {},
        context: { workingDirectory: directory },
      }),
      /host request failed/u,
    );
    assert.equal(await host.exited, 1);
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});
