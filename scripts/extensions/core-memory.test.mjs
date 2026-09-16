import assert from "node:assert/strict";
import { test } from "node:test";

import { createExtensionApi } from "../../src-tauri/resources/extension-host/extension-api.mjs";

test("memory API exposes only scoped topic operations", () => {
  const { api } = createExtensionApi({
    id: "com.example.memory",
    manifest: { apiLevel: "stable" },
  });

  assert.equal(typeof api.memory?.list, "function");
  assert.equal(typeof api.memory?.read, "function");
  assert.equal(typeof api.memory?.write, "function");
  assert.equal(typeof api.memory?.archive, "function");
  assert.throws(() => api.memory.list({ scope: "other" }), /core_request_failed/u);
  assert.throws(
    () => api.memory.write({ scope: "global", path: "/tmp/index", content: "x" }),
    /core_request_failed/u,
  );
  assert.throws(
    () => api.memory.write({ scope: "global", topicId: "id", content: "x" }),
    /core_request_failed/u,
  );
});
