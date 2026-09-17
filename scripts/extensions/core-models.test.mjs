import assert from "node:assert/strict";
import { test } from "node:test";

import { createExtensionApi } from "../../src-tauri/resources/extension-host/extension-api.mjs";
import { negotiateCapabilities } from "../../src-tauri/resources/extension-host/extension-api-capabilities.mjs";
import { LIMITS, MODEL_FINISH_REASONS, OPTIONAL_CAPABILITIES } from "../../src-tauri/resources/extension-host/contract.mjs";

negotiateCapabilities(OPTIONAL_CAPABILITIES);

test("models API exposes the generated bounds and validates before transport", () => {
  const { api } = createExtensionApi({
    id: "com.example.models",
    manifest: { apiLevel: "stable" },
  });

  assert.equal(typeof api.models?.list, "function");
  assert.deepEqual(MODEL_FINISH_REASONS, ["stop", "length", "contentFilter"]);
  assert.throws(
    () => api.models.generate({
      prompt: "ok",
      maxOutputTokens: LIMITS.maxModelOutputTokens + 1,
    }),
    /core_request_failed/u,
  );
  assert.throws(
    () => api.models.generate({ prompt: "x".repeat(LIMITS.maxModelPromptBytes + 1) }),
    /core_request_failed/u,
  );
});
