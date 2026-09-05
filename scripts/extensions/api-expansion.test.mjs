import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
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
import { loadExtensionWithApi, resetExtensions } from "../../src-tauri/resources/extension-host/loader.mjs";
import { createLegacyHostContext } from "./fixtures/legacy-host.mjs";

test("oversized contributions fail at registration without publishing any tools", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-contribution-budget-"));
  const mainPath = join(directory, "index.mjs");
  await writeFile(mainPath, `export default function(api) {
    api.registerTool({name:"probe", description:"Probe", parameters:{type:"object"}, execute:()=>({content:"ok"})});
    for (let index = 0; index < 64; index++) api.registerResource({
      id:"r"+index, name:"🦫".repeat(100), description:"🦫".repeat(2000),
      type:"file", path:"🦫".repeat(4096)
    });
  }`);
  try {
    await resetExtensions();
    const result = await loadExtensionWithApi({
      id: "sample.oversized", mainPath, manifest: {apiLevel:"stable"},
    }, createExtensionApi);
    assert.equal(result.error, "load_failed");
    assert.equal(result.diagnostic.code, "registration_failed");
    const { callExtensionTool } = await import("../../src-tauri/resources/extension-host/loader.mjs");
    await assert.rejects(callExtensionTool("sample.oversized.probe", {}, {workingDirectory:directory}), /tool_not_found/);
  } finally {
    await resetExtensions();
    await rm(directory, {recursive:true, force:true});
  }
});

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

  assert.deepEqual(api.capabilities, [...CAPABILITIES, "skills", "resources", "richToolResults"]);
  assert.equal(Object.isFrozen(api.capabilities), true);
  assert.throws(() => api.capabilities.push("skills"), TypeError);
  assert.deepEqual(OPTIONAL_CAPABILITIES, ["skills", "resources", "richToolResults"]);
  assert.equal(api.capabilities.includes("skills"), true);
  assert.equal(api.capabilities.includes("resources"), true);
  assert.equal(api.capabilities.includes("richToolResults"), true);
});

test("une extension récente reste compatible avec un Hôte historique seulement si elle garde les capacités", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-legacy-host-"));
  const guarded = join(directory, "guarded.mjs");
  const unguarded = join(directory, "unguarded.mjs");
  const specification = (mainPath) => ({
    id: "com.example.legacy-host",
    mainPath,
    manifest: { apiLevel: "stable" },
  });
  await writeFile(guarded, `export default function (api) {
    if (api.capabilities?.includes("skills") && api.capabilities?.includes("resources")) {
      api.registerSkill({ id: "guide", name: "Guide", description: "Description", path: "SKILL.md" });
      api.registerResource({ id: "reference", name: "Reference", description: "Description", type: "text", path: "reference.txt" });
    }
  }`);
  await writeFile(unguarded, `export default function (api) {
    api.registerSkill({ id: "guide", name: "Guide", description: "Description", path: "SKILL.md" });
  }`);
  try {
    await resetExtensions();
    const compatible = await loadExtensionWithApi(specification(guarded), createLegacyHostContext);
    assert.equal(compatible.error, undefined);
    assert.deepEqual(compatible.contributions.skills, []);
    assert.deepEqual(compatible.contributions.resources, []);

    const incompatible = await loadExtensionWithApi(specification(unguarded), createLegacyHostContext);
    assert.equal(incompatible.error, "load_failed");
    assert.equal(incompatible.diagnostic.code, "activation_failed");
    assert.equal("message" in incompatible.diagnostic, false);
  } finally {
    await resetExtensions();
    await rm(directory, { recursive: true, force: true });
  }
});

test("skills et ressources sont capturés une seule fois avant validation", () => {
  const { api, skills, resources } = createExtensionApi({
    id: "com.example.contributions",
    manifest: { apiLevel: "stable" },
  });
  let reads = 0;
  const skill = {
    id: "reference",
    get name() { reads += 1; return "Reference"; },
    description: "Guidance",
    path: "skills/reference/SKILL.md",
  };

  api.registerSkill(skill);
  skill.description = "Changed after capture";
  api.registerResource({
    id: "preview",
    name: "Preview",
    description: "Image preview",
    type: "image",
    path: "resources/preview.png",
  });

  assert.equal(reads, 1);
  assert.deepEqual(JSON.parse(JSON.stringify(skills)), [{
    id: "reference",
    name: "Reference",
    description: "Guidance",
    path: "skills/reference/SKILL.md",
  }]);
  assert.deepEqual(JSON.parse(JSON.stringify(resources)), [{
    id: "preview",
    name: "Preview",
    description: "Image preview",
    type: "image",
    path: "resources/preview.png",
  }]);
});

test("l’Hôte borne et déduplique les déclarations de skills et ressources", () => {
  const { api } = createExtensionApi({
    id: "com.example.bounds",
    manifest: { apiLevel: "stable" },
  });
  for (let index = 0; index < LIMITS.maxSkillsPerExtension; index += 1) {
    api.registerSkill({
      id: `skill-${index}`,
      name: "Skill",
      description: "Description",
      path: `skills/${index}/SKILL.md`,
    });
  }
  assert.throws(() => api.registerSkill({
    id: "one-too-many",
    name: "Skill",
    description: "Description",
    path: "skills/overflow/SKILL.md",
  }), /invalid_skill/u);
  assert.throws(() => api.registerSkill({
    id: "skill-0",
    name: "Duplicate",
    description: "Description",
    path: "skills/duplicate/SKILL.md",
  }), /invalid_skill/u);

  const { api: resourceApi } = createExtensionApi({
    id: "com.example.resources",
    manifest: { apiLevel: "stable" },
  });
  for (let index = 0; index < LIMITS.maxResourcesPerExtension; index += 1) {
    resourceApi.registerResource({
      id: `resource-${index}`,
      name: "Resource",
      description: "Description",
      type: "text",
      path: `resources/${index}.txt`,
    });
  }
  assert.throws(() => resourceApi.registerResource({
    id: "one-too-many",
    name: "Resource",
    description: "Description",
    type: "text",
    path: "resources/overflow.txt",
  }), /invalid_resource/u);
});

test("l’Hôte refuse les chemins de contribution ambigus", () => {
  const { api } = createExtensionApi({
    id: "com.example.paths",
    manifest: { apiLevel: "stable" },
  });
  for (const path of ["/absolute.md", "../parent.md", "dir\\file.md", "C:/file.md", "dir\0file.md"]) {
    assert.throws(() => api.registerSkill({
      id: `skill-${path.length}`,
      name: "Skill",
      description: "Description",
      path,
    }), /invalid_skill/u);
  }
  assert.throws(() => api.registerSkill({
    id: "root-injection",
    name: "Skill",
    description: "Description",
    path: "skills/guide/SKILL.md",
    root: "/untrusted",
  }), /invalid_skill/u);
});

test("un même identifiant local reste indépendant de son extension", () => {
  const first = createExtensionApi({ id: "com.example.first", manifest: { apiLevel: "stable" } });
  const second = createExtensionApi({ id: "com.example.second", manifest: { apiLevel: "stable" } });
  const skill = { id: "guide", name: "Guide", description: "Description", path: "SKILL.md" };

  first.api.registerSkill(skill);
  second.api.registerSkill(skill);

  assert.equal(first.skills[0].id, second.skills[0].id);
});

test("la fixture API expansion installable enregistre deux outils, son skill et ses ressources", async () => {
  const root = resolve("src-tauri/tests/fixtures/extensions/api-expansion");
  await resetExtensions();
  try {
    const loaded = await loadExtensionWithApi({
      id: "acceptance.api.expansion",
      mainPath: join(root, "index.ts"),
      manifest: { apiLevel: "stable" },
    }, createExtensionApi);

    assert.equal(loaded.error, undefined);
    assert.deepEqual(
      loaded.contributions.tools.map(({ name }) => name),
      ["acceptance.api.expansion.catalog_probe", "acceptance.api.expansion.produce_artifacts"],
    );
    assert.deepEqual(loaded.contributions.skills.map(({ id }) => id), ["reference-skill"]);
    assert.deepEqual(
      loaded.contributions.resources.map(({ id }) => id),
      ["reference", "preview"],
    );
  } finally {
    await resetExtensions();
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
