import assert from "node:assert/strict";
import { test } from "node:test";

import { snapshotContribution } from "../../src-tauri/resources/extension-host/contribution-snapshot.mjs";

test("scripts/extensions/contribution-snapshot.test.mjs: captures getters once without a prototype", () => {
  let reads = 0;
  const input = {
    id: "guide",
    get name() { reads += 1; return "Guide"; },
    description: "Read this",
    path: "skills/guide.md",
  };

  const snapshot = snapshotContribution(input);
  input.description = "changed";

  assert.equal(reads, 1);
  assert.equal(Object.getPrototypeOf(snapshot), null);
  assert.equal(snapshot.description, "Read this");
});
