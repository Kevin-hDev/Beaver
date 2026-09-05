import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { LIMITS } from "../../src-tauri/resources/extension-host/contract.mjs";
import { createHost, resetAndLoad } from "./host-test-client.mjs";
import { hostScript } from "./office-test-helpers.mjs";
import { assertProtocolResultFits, MAX_REQUEST_ID_CHARS } from "../../src-tauri/resources/extension-host/protocol-output.mjs";

test("registration budget uses the real wire limit, including the worst-case envelope", () => {
  const result = { contributions: { description: "" } };
  const overhead = Buffer.byteLength(JSON.stringify({
    jsonrpc: "2.0", id: "\u0000".repeat(MAX_REQUEST_ID_CHARS), result,
  }) + "\n");
  result.contributions.description = "x".repeat(LIMITS.maxMessageBytes - overhead - 1);
  assert.doesNotThrow(() => assertProtocolResultFits(result));
  result.contributions.description += "x";
  assert.throws(() => assertProtocolResultFits(result), /message_too_large/);
});

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
    await resetAndLoad(host, [{
        id: "com.beaver.large-output",
        mainPath: source,
        manifest: { apiLevel: "stable" },
      }]);
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
