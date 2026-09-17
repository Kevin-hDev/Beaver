import assert from "node:assert/strict";
import { test } from "node:test";

import { createExtensionApi } from "../../src-tauri/resources/extension-host/extension-api.mjs";
import { negotiateCapabilities } from "../../src-tauri/resources/extension-host/extension-api-capabilities.mjs";
import { OPTIONAL_CAPABILITIES } from "../../src-tauri/resources/extension-host/contract.mjs";

negotiateCapabilities(OPTIONAL_CAPABILITIES);

test("automations API exposes only the attributed CRUD surface", () => {
  const { api } = createExtensionApi({
    id: "com.example.automations",
    manifest: { apiLevel: "stable" },
  });

  assert.deepEqual(
    Object.keys(api.automations).sort(),
    ["create", "delete", "list", "setActive", "update"],
  );
  assert.throws(
    () => api.automations.create({ name: "Wake", prompt: "Run", schedule: {}, active: true }),
    /core_request_failed/u,
  );
  assert.throws(
    () => api.automations.setActive("id", 1, "yes"),
    /core_request_failed/u,
  );
  assert.throws(
    () => api.automations.update("id", "1", {}),
    /core_request_failed/u,
  );
});
