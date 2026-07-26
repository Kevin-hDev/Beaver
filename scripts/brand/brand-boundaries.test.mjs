import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  checkContracts,
  formatReport,
  scanEntries,
  validateTrackedPath,
} from "./brand-boundaries.mjs";
import {
  COMPATIBILITY_CONTRACTS,
  EXPECTED_INTERNAL_REFERENCE_COUNTS,
} from "./brand-boundaries-contracts.mjs";
import { loadTrackedEntries } from "./brand-boundaries-repository.mjs";
import { EXPECTED_VISIBLE_REFERENCE_CONTRACT } from "./brand-boundaries-visible-contracts.mjs";

const PROJECT_ROOT = fileURLToPath(new URL("../../", import.meta.url));

function countByValue(findings) {
  const counts = {};
  for (const finding of findings) {
    counts[finding.value] = (counts[finding.value] ?? 0) + 1;
  }
  return counts;
}

function visibleContracts(findings) {
  return findings.map(({ file, value, contextHash }) => [
    file,
    value,
    contextHash,
  ]);
}

function visibleContract(findings) {
  const references = visibleContracts(findings);
  return {
    count: references.length,
    sha256: createHash("sha256")
      .update(JSON.stringify(references), "utf8")
      .digest("hex"),
  };
}

test("le contexte visible change même si le nombre d'occurrences reste égal", () => {
  const first = scanEntries([{ file: "sample.ts", content: 'title = "CL-GO"' }]);
  const second = scanEntries([{ file: "sample.ts", content: 'tooltip = "CL-GO"' }]);
  assert.notEqual(first.visible[0].contextHash, second.visible[0].contextHash);
});

test("le dépôt respecte toutes les frontières de marque", () => {
  const failures = checkContracts((file) =>
    readFileSync(resolve(PROJECT_ROOT, validateTrackedPath(file)), "utf8"),
  );
  const report = scanEntries(loadTrackedEntries(PROJECT_ROOT));
  const output = formatReport(report);

  console.log(output);
  assert.deepEqual(failures, [], failures.join("\n"));
  assert.deepEqual(report.unknown, [], output);
  assert.deepEqual(
    countByValue(report.internal),
    EXPECTED_INTERNAL_REFERENCE_COUNTS,
  );
  assert.deepEqual(
    visibleContract(report.visible),
    EXPECTED_VISIBLE_REFERENCE_CONTRACT,
  );
});
