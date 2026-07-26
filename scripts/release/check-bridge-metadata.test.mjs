import assert from "node:assert/strict";
import test from "node:test";

import { validateBridgeMetadata } from "./check-bridge-metadata.mjs";

function validMetadata() {
  return {
    packageJson: {
      name: "cl-go-dash",
      version: "1.0.2",
    },
    packageLock: {
      name: "cl-go-dash",
      version: "1.0.2",
      packages: {
        "": {
          name: "cl-go-dash",
          version: "1.0.2",
        },
      },
    },
    cargoToml: '[package]\nname = "cl-go-dash"\nversion = "1.0.2"\n',
    cargoLock: '[[package]]\nname = "cl-go-dash"\nversion = "1.0.2"\n',
    tauriConfig: {
      productName: "CL-GO",
      version: "1.0.2",
      identifier: "com.clgo.dash",
    },
  };
}

test("accepte uniquement une version-pont cohérente", () => {
  assert.deepEqual(validateBridgeMetadata(validMetadata(), "v1.0.2"), {
    tag: "v1.0.2",
    version: "1.0.2",
  });
});

test("accepte les fins de ligne Windows dans les fichiers Cargo", () => {
  const metadata = validMetadata();
  metadata.cargoToml = metadata.cargoToml.replaceAll("\n", "\r\n");
  metadata.cargoLock = metadata.cargoLock.replaceAll("\n", "\r\n");

  assert.deepEqual(validateBridgeMetadata(metadata, "v1.0.2"), {
    tag: "v1.0.2",
    version: "1.0.2",
  });
});

test("refuse une version différente dans chaque source", () => {
  const mutations = [
    (value) => (value.packageJson.version = "1.0.1"),
    (value) => (value.packageLock.version = "1.0.1"),
    (value) => (value.packageLock.packages[""].version = "1.0.1"),
    (value) => (value.cargoToml = value.cargoToml.replace("1.0.2", "1.0.1")),
    (value) => (value.cargoLock = value.cargoLock.replace("1.0.2", "1.0.1")),
    (value) => (value.tauriConfig.version = "1.0.1"),
  ];

  for (const mutate of mutations) {
    const metadata = validMetadata();
    mutate(metadata);
    assert.throws(() => validateBridgeMetadata(metadata, "v1.0.2"));
  }
});

test("refuse de modifier les identifiants historiques", () => {
  const mutations = [
    (value) => (value.packageJson.name = "beaver"),
    (value) => (value.packageLock.name = "beaver"),
    (value) => (value.packageLock.packages[""].name = "beaver"),
    (value) => (value.cargoToml = value.cargoToml.replace("cl-go-dash", "beaver")),
    (value) => (value.cargoLock = value.cargoLock.replace("cl-go-dash", "beaver")),
    (value) => (value.tauriConfig.productName = "Beaver"),
    (value) => (value.tauriConfig.identifier = "com.beaver.app"),
  ];

  for (const mutate of mutations) {
    const metadata = validMetadata();
    mutate(metadata);
    assert.throws(() => validateBridgeMetadata(metadata, "v1.0.2"));
  }
});

test("refuse les tags hors de la série corrective 1.0", () => {
  for (const tag of ["", "1.0.2", "v1.0.1", "v1.1.0", "v1.0.02", "../v1.0.2"]) {
    assert.throws(() => validateBridgeMetadata(validMetadata(), tag));
  }
});
