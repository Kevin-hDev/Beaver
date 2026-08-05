import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, statSync } from "node:fs";
import { resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("../../", import.meta.url));
const MANIFEST_PATH = resolve(
  ROOT,
  "scripts/migration/cl-go-v1.0.2-profile.json",
);
const MAX_MANIFEST_BYTES = 128 * 1024;
const EXPECTED_BASELINE_ATTESTATION =
  "737aa2b543328885e163331f81fc343eb2b8cb8d62a0ae5a0a52b00c58cc8585";

function loadManifest() {
  assert.ok(statSync(MANIFEST_PATH).size <= MAX_MANIFEST_BYTES);
  return JSON.parse(readFileSync(MANIFEST_PATH, "utf8"));
}

function baselineAttestation(manifest) {
  const historicalContracts = {
    baseline: manifest.baseline,
    domains: manifest.domains.map(({ id, contracts }) => ({
      id,
      contracts: contracts.filter(
        ({ scope }) => (scope ?? "baseline") === "baseline",
      ),
    })),
  };
  return createHash("sha256")
    .update(JSON.stringify(historicalContracts))
    .digest("hex");
}

test("l'attestation locale des contrats historiques reste intacte", () => {
  assert.equal(baselineAttestation(loadManifest()), EXPECTED_BASELINE_ATTESTATION);
});
