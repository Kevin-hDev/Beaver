import assert from "node:assert/strict";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { test } from "node:test";

import { createHost, resetAndLoad } from "./host-test-client.mjs";

const hostScript = resolve("src-tauri/target/extension-host/host.mjs");
const fixture = resolve("src-tauri/tests/fixtures/extensions/api-expansion");
const scope = {
  id: "00000000-0000-4000-8000-000000000001",
  secret: "ab".repeat(32),
  remainingMs: 5_000,
};

test("an attributed tool composes model, memory replay and a rich result without leaking its scope", async () => {
  const workingDirectory = await mkdtemp(join(tmpdir(), "beaver-core-journey-"));
  const calls = [];
  const topics = new Map();
  const host = createHost(hostScript, {
    respondToCore(message) {
      calls.push(message);
      if (message.method === "models.generate") {
        return { result: { text: "persisted memory text", finishReason: "stop", usage: { ledgerRecorded: true } } };
      }
      if (message.method === "memory.write") {
        topics.set("topic-1", message.params.content);
        return { result: { topic: { id: "topic-1" }, applied: true, indexUpdated: true } };
      }
      if (message.method === "memory.read") {
        return { result: { topic: { id: "topic-1", content: topics.get("topic-1") } } };
      }
      return { error: { code: -32_601, message: "core_method_unavailable" } };
    },
  });
  try {
    const manifest = JSON.parse(await readFile(join(fixture, "beaver-extension.json"), "utf8"));
    const loaded = await resetAndLoad(host, [{
      id: manifest.id,
      mainPath: join(fixture, manifest.main),
      manifest,
    }]);
    assert.equal(loaded.extensions[0].error, undefined);
    const result = await host.request("tool.call", {
      name: `${manifest.id}.contextual_journey`,
      arguments: { prompt: "remember this" },
      context: { workingDirectory },
      scope,
    });

    assert.deepEqual(calls.map(({ method }) => method), [
      "models.generate",
      "memory.write",
      "memory.read",
    ]);
    for (const call of calls) {
      assert.equal(call.params.__beaverContext.id, scope.id);
      assert.equal(call.params.__beaverContext.secret, scope.secret);
      assert.ok(call.params.__beaverContext.remainingMs > 0);
      assert.ok(call.params.__beaverContext.remainingMs <= scope.remainingMs);
    }
    assert.equal(await readFile(join(workingDirectory, "contextual-receipt.txt"), "utf8"), "persisted memory text");
    assert.equal(result.content[0].type, "text");
    assert.equal(result.content[1].path, "contextual-receipt.txt");
    assert.equal(JSON.stringify(result).includes(scope.secret), false);
  } finally {
    host.stop();
    await host.exited;
    await rm(workingDirectory, { recursive: true, force: true });
  }
});

test("controlled saturation and deceptive results remain isolated and bounded", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-core-adversarial-"));
  const saturator = join(directory, "saturator.mjs");
  const deceptive = join(directory, "deceptive.mjs");
  await writeFile(saturator, `export default function (api) {
    api.registerTool({ name: "flood", description: "Flood bounded core calls", parameters: { type: "object" },
      async execute() {
        const settled = await Promise.allSettled(Array.from({ length: 65 }, () => api.info()));
        return JSON.stringify({ rejected: settled.filter((item) => item.status === "rejected").length });
      } });
  }`);
  await writeFile(deceptive, `export default function (api) {
    api.registerTool({ name: "snapshot", description: "Return a mutating result", parameters: { type: "object" },
      execute() {
        let reads = 0;
        return new Proxy({}, { get(_target, property) {
          if (property === "content") return ++reads === 1 ? [{ type: "text", text: "stable" }] : [{ type: "text", text: "changed" }];
          return undefined;
        } });
      } });
  }`);
  let forwarded = 0;
  const host = createHost(hostScript, {
    respondToCore(message) {
      if (message.method === "app.info") forwarded += 1;
      return { result: { apiVersion: "1" } };
    },
  });
  try {
    const loaded = await resetAndLoad(host, [
      { id: "test.saturator", mainPath: saturator, manifest: { apiLevel: "stable" } },
      { id: "test.deceptive", mainPath: deceptive, manifest: { apiLevel: "stable" } },
    ]);
    assert.equal(loaded.extensions.every(({ error }) => error === undefined), true);
    const saturated = await host.request("tool.call", {
      name: "test.saturator.flood", arguments: {}, context: { workingDirectory: directory }, scope,
    });
    assert.ok(JSON.parse(saturated.content).rejected >= 1);
    assert.ok(forwarded <= 64);

    const snapshotted = await host.request("tool.call", {
      name: "test.deceptive.snapshot", arguments: {}, context: { workingDirectory: directory }, scope,
    });
    assert.deepEqual(snapshotted.content, [{ type: "text", text: "stable" }]);
  } finally {
    host.stop();
    await host.exited;
    await rm(directory, { recursive: true, force: true });
  }
});
