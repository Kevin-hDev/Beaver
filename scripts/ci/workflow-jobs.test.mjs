import assert from "node:assert/strict";
import test from "node:test";

import { jobSection } from "./workflow-jobs.mjs";

test("nested keys do not end a root-level job section", () => {
  const workflow = `frontend:
  steps:
    - run: npm run test:release-workflow
_release:
  steps:
    - run: npm publish
`;

  assert.match(jobSection(workflow, "frontend"), /npm run test:release-workflow/u);
  assert.doesNotMatch(jobSection(workflow, "frontend"), /npm publish/u);
});
