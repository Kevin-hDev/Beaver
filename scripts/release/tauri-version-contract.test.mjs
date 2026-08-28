import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { assertTauriVersionContract } from "./tauri-version-contract.mjs";

const packageJson = {
  devDependencies: { "@tauri-apps/cli": "2.11.4" },
};
const cargoToml = `
[build-dependencies]
tauri-build = { version = "=2.6.3", features = [] }

[dependencies]
tauri = { version = "=2.11.5", features = ["tray-icon"] }
`;
const cargoLock = `
[[package]]
name = "tauri"
version = "2.11.5"

[[package]]
name = "tauri-build"
version = "2.6.3"
`;

function verify(overrides = {}) {
  return assertTauriVersionContract({ packageJson, cargoToml, cargoLock, ...overrides });
}

test("accepte uniquement la paire Tauri vérifiée", () => {
  assert.doesNotThrow(() => verify());
});

test("le dépôt respecte réellement le contrat Tauri", () => {
  assert.doesNotThrow(() =>
    assertTauriVersionContract({
      packageJson: readFileSync("package.json", "utf8"),
      cargoToml: readFileSync("src-tauri/Cargo.toml", "utf8"),
      cargoLock: readFileSync("src-tauri/Cargo.lock", "utf8"),
    }),
  );
});

test("refuse une plage npm", () => {
  assert.throws(
    () => verify({ packageJson: { devDependencies: { "@tauri-apps/cli": "^2.11.4" } } }),
    /Tauri version contract failed/,
  );
});

test("refuse une version Rust différente dans le manifeste ou le verrou", () => {
  assert.throws(
    () => verify({ cargoToml: cargoToml.replace("=2.11.5", "2") }),
    /Tauri version contract failed/,
  );
  assert.throws(
    () => verify({ cargoLock: cargoLock.replace("2.11.5", "2.11.4") }),
    /Tauri version contract failed/,
  );
});

test("borne les fichiers analysés et masque les détails internes", () => {
  assert.throws(
    () => verify({ cargoLock: "x".repeat(2_097_153) }),
    (error) => error.message === "Tauri version contract failed",
  );
});
