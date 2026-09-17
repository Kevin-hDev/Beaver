import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { readRegularTextSync } from "../file-system/regular-file.mjs";

const ROOT = fileURLToPath(new URL("../../", import.meta.url));
const MANIFEST_PATH = resolve(
  ROOT,
  "scripts/migration/cl-go-v1.0.2-profile.json",
);
const MAX_MANIFEST_BYTES = 128 * 1024;
const EXPECTED_BASELINE_ATTESTATION =
  "351f624c4aa41897734431e4a1fa9320fa5d9bbc1c016343d52d66b486953a1e";

function loadManifest() {
  return JSON.parse(readRegularTextSync(MANIFEST_PATH, MAX_MANIFEST_BYTES));
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
