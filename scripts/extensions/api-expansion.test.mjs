import assert from "node:assert/strict";
import { test } from "node:test";

import {
  CAPABILITIES,
  CONTRIBUTION_TYPES,
  LIMITS,
  OPTIONAL_CAPABILITIES,
  RESOURCE_TYPES,
  RESULT_BLOCK_TYPES,
} from "../../src-tauri/resources/extension-host/contract.mjs";
import { createExtensionApi } from "../../src-tauri/resources/extension-host/extension-api.mjs";

test("API 1 historique charge sans lire api.capabilities", () => {
  const { api } = createExtensionApi({
    id: "com.example.legacy",
    manifest: { apiLevel: "stable" },
  });

  assert.equal(api.id, "com.example.legacy");
  assert.equal("capabilities" in api, true);
});

test("api.capabilities est une copie gelée des capacités déjà utilisables", () => {
  const { api } = createExtensionApi({
    id: "com.example.capabilities",
    manifest: { apiLevel: "stable" },
  });

  assert.deepEqual(api.capabilities, CAPABILITIES);
  assert.equal(Object.isFrozen(api.capabilities), true);
  assert.throws(() => api.capabilities.push("skills"), TypeError);
  assert.deepEqual(OPTIONAL_CAPABILITIES, ["skills", "resources", "richToolResults"]);
  for (const capability of OPTIONAL_CAPABILITIES) {
    assert.equal(api.capabilities.includes(capability), false);
  }
});

test("le contrat fixe les formes et bornes R0 sans les activer", () => {
  assert.deepEqual(RESULT_BLOCK_TYPES, ["text", "file"]);
  assert.deepEqual(RESOURCE_TYPES, ["text", "image", "file"]);
  assert.deepEqual(CONTRIBUTION_TYPES, ["tool", "event", "ui", "skill", "resource"]);
  assert.equal(LIMITS.maxSkillsPerExtension, 32);
  assert.equal(LIMITS.maxExtensionNameChars, 100);
  assert.equal(LIMITS.maxExtensionTextChars, 2_000);
  assert.equal(LIMITS.maxResourcesPerExtension, 64);
  assert.equal(LIMITS.maxResultBlocks, 16);
  assert.equal(LIMITS.maxResultFiles, 8);
  assert.equal(LIMITS.maxResultTextBytes, 524_288);
  assert.equal(LIMITS.maxTextResourceBytes, 262_144);
  assert.equal(LIMITS.maxResourceFileBytes, 20_971_520);
  assert.equal(LIMITS.maxResultBytes, 20_971_520);
  assert.equal(LIMITS.maxPathChars, 4_096);
  assert.equal(LIMITS.maxParallelEphemeralArtifactBytes, 67_108_864);
  assert.equal(LIMITS.maxMultimodalPreviewsPerContinuation, 8);
});

test("les métadonnées humaines Node comptent les valeurs scalaires Unicode", () => {
  const { api } = createExtensionApi({
    id: "com.example.unicode",
    manifest: { apiLevel: "stable" },
  });

  api.registerTool({
    name: "unicode",
    description: "🦫".repeat(2_000),
    parameters: { type: "object" },
    execute: () => "ok",
  });
  assert.throws(() => api.registerTool({
    name: "unicode-overflow",
    description: "🦫".repeat(2_001),
    parameters: { type: "object" },
    execute: () => "ok",
  }), /invalid_tool/u);
});
