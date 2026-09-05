import assert from "node:assert/strict";
import { test } from "node:test";
import { createExtensionApi } from "../../src-tauri/resources/extension-host/extension-api.mjs";
import { validRelativePath } from "../../src-tauri/resources/extension-host/contribution-validation.mjs";

test("skill registration accepts only the two manifest basenames", () => {
  const { api } = createExtensionApi({ id: "sample", manifest: { apiLevel: "stable" } });
  for (const path of ["guide.txt", "skills/guide.md", "Skill.md"]) {
    assert.throws(() => api.registerSkill({ id: "guide", name: "Guide", description: "Guide", path }));
  }
  api.registerSkill({ id: "root", name: "Guide", description: "Guide", path: "SKILL.md" });
  api.registerSkill({ id: "nested", name: "Guide", description: "Guide", path: "skills/skill.md" });
});

test("relative paths reject Windows reserved names and C1 controls", () => {
  for (const path of ["CON.txt", "dir/LPT9 .md", "COM¹", "LPT².txt", "COM³... ", "a\u0085b"]) {
    assert.equal(validRelativePath(path), false, path);
  }
  assert.equal(validRelativePath("skills/guide/SKILL.md"), true);
  assert.equal(validRelativePath("resources/COM10.txt"), true);
});
