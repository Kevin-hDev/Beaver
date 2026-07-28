import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { LIMITS } from "../../src-tauri/resources/extension-host/contract.mjs";
import { createHost } from "./host-test-client.mjs";
import { hostScript } from "./office-test-helpers.mjs";

test("truncates oversized third-party tool output without stopping the host", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-extension-output-"));
  const source = join(directory, "index.ts");
  await writeFile(
    source,
    `export default function (api: any) {
      api.registerTool({
        name: "large",
        description: "Return an oversized result",
        parameters: { type: "object" },
        execute() { return "\\"\\\\u0001".repeat(800000); }
      });
    }`,
    { mode: 0o600 },
  );
  const host = createHost(hostScript);
  try {
    await host.request("host.sync", {
      extensions: [{
        id: "com.beaver.large-output",
        mainPath: source,
        manifest: { apiLevel: "stable" },
      }],
    });
    const result = await host.request("tool.call", {
      name: "com.beaver.large-output.large",
      arguments: {},
      context: { workingDirectory: directory },
    });

    assert.equal(result.truncated, true);
    assert.ok(Buffer.byteLength(JSON.stringify(result), "utf8") < LIMITS.maxMessageBytes);
    assert.equal((await host.request("host.hello", {})).apiVersion, "1");
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});
