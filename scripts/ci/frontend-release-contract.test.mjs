import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import { jobSection } from "./workflow-jobs.mjs";

test("pull request CI runs the application version contract", async () => {
  const workflow = await readFile(".github/workflows/ci.yml", "utf8");

  assert.match(jobSection(workflow, "frontend"), /npm run test:release-workflow(?:\r?\n|$)/u);
});

test("a following underscore-prefixed job cannot satisfy the frontend contract", () => {
  const workflow = `jobs:
  frontend:
    steps:
      - run: npm test
  _release:
    steps:
      - run: npm run test:release-workflow
`;

  assert.doesNotMatch(jobSection(workflow, "frontend"), /npm run test:release-workflow/u);
});
