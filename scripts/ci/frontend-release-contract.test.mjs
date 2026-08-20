import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

function frontendJob(workflow) {
  const start = workflow.indexOf("  frontend:\n");
  assert.notEqual(start, -1);
  const remaining = workflow.slice(start);
  const end = remaining.search(/\n  [a-z][a-z0-9_-]*:\s*\n/iu);
  return end < 0 ? remaining : remaining.slice(0, end);
}

test("pull request CI runs the application version contract", async () => {
  const workflow = await readFile(".github/workflows/ci.yml", "utf8");

  assert.match(frontendJob(workflow), /npm run test:release-workflow(?:\r?\n|$)/u);
});
