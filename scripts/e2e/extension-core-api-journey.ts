import assert from "node:assert/strict";

const CONTEXTUAL_TOOL = "acceptance.api.expansion.contextual_journey";

export function assertContextualCapabilityProjection(toolNames: string[]): void {
  const expected = [
    "acceptance.api.expansion.catalog_probe",
  ];
  if (process.platform !== "linux") expected.push(CONTEXTUAL_TOOL);
  expected.push("acceptance.api.expansion.produce_artifacts");
  assert.deepEqual(toolNames, expected);
}
