import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { test } from "node:test";

import { createHost, resetAndLoad } from "./host-test-client.mjs";

const root = resolve(".");
const fixture = join(root, "scripts", "extensions", "fixtures", "core-api");
const manifest = JSON.parse(await readFile(join(fixture, "beaver-extension.json"), "utf8"));

test("the documented contextual API example loads through Jiti and uses the core facade", async () => {
  const calls = [];
  const host = createHost(resolve("src-tauri/target/extension-host/host.mjs"), {
    respondToCore(message) {
      calls.push({ method: message.method, params: message.params });
      const results = {
        "models.generate": { text: "summary", finishReason: "stop", usage: { ledgerRecorded: true } },
        "memory.write": { topic: { id: "topic-1", title: "Example", updatedAt: "2026-09-16T00:00:00Z", content: "remember" }, applied: true, indexUpdated: true },
        "automations.create": { id: "auto-1", revision: 1, name: "Extension follow-up", description: null, prompt: "later", schedule: {}, active: false },
        "subagents.spawn": { id: "child-1", type: "explorer", status: "running" },
      };
      return message.method in results
        ? { result: results[message.method] }
        : { error: { code: -32_601, message: "core_method_unavailable" } };
    },
  });

  try {
    assert.equal(manifest.main, "./index.ts");
    const loaded = await resetAndLoad(host, [{
      id: manifest.id,
      mainPath: join(fixture, manifest.main),
      manifest,
    }]);
    assert.equal(loaded.extensions[0].error, undefined);
    assert.deepEqual(
      loaded.extensions[0].contributions.tools.map((tool) => tool.name),
      ["generate_summary", "remember_topic", "propose_wakeup", "start_explorer", "observed_turns"]
        .map((name) => `${manifest.id}.${name}`),
    );

    const call = (name, arguments_) => host.request("tool.call", {
      name: `${manifest.id}.${name}`,
      arguments: arguments_,
      context: { workingDirectory: fixture },
    });
    assert.match((await call("generate_summary", { prompt: "summarize" })).content, /summary/u);
    assert.match((await call("remember_topic", { content: "remember" })).content, /topic-1/u);
    assert.match((await call("propose_wakeup", { prompt: "later" })).content, /requiresUserApproval/u);
    assert.match((await call("start_explorer", { prompt: "inspect" })).content, /child-1/u);
    assert.equal(calls.find(({ method }) => method === "automations.create").params.active, undefined);

    await host.request("event.emit", { event: "session.turn.completed", payload: { sessionId: "s1" } });
    assert.equal((await call("observed_turns", {})).content, "1");
    assert.deepEqual(await host.request("tool.intercept", {
      extensionId: manifest.id,
      call: { toolName: "bash", effect: "process", mode: "auto" },
    }), { decision: "deny", reason: "example_process_guard" });
  } finally {
    host.stop();
    await host.exited;
  }
});
