import assert from "node:assert/strict";
import { test } from "node:test";

import { createExtensionApi } from "../../src-tauri/resources/extension-host/extension-api.mjs";

test("subagents API exposes only owned orchestration methods", () => {
  const { api } = createExtensionApi({
    id: "com.example.subagents",
    manifest: { apiLevel: "stable" },
  });

  assert.deepEqual(
    Object.keys(api.subagents).sort(),
    ["cancel", "get", "list", "send", "spawn"],
  );
  assert.throws(() => api.subagents.spawn("invalid", "work"), /core_request_failed/u);
  assert.throws(() => api.subagents.send("child", ""), /core_request_failed/u);
});
