import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  MAX_INVENTORY_ENTRIES,
  MAX_PROOF_BYTES,
  canonicalizeDataDirectory,
  collectNativeUpgradeProof,
  serializeNativeUpgradeProof,
} from "./native-upgrade-proof.mjs";

async function makeDisposableProfile() {
  const homeDirectory = await mkdtemp(join(tmpdir(), "beaver-native-proof-home-"));
  const dataDirectory = join(homeDirectory, ".local", "share", "cl-go-dash");
  const modelsDirectory = join(homeDirectory, ".ollama", "models");
  await mkdir(join(dataDirectory, "ollama-bundle"), { recursive: true });
  await mkdir(modelsDirectory, { recursive: true });
  await writeFile(join(dataDirectory, "ollama-bundle", "VERSION"), "1.1.2\n");
  await writeFile(join(modelsDirectory, "manifest.json"), "model metadata\n");
  return { homeDirectory, dataDirectory, modelsDirectory };
}

function confirmed(profile) {
  return {
    ...profile,
    confirmDisposableProfile: true,
    confirmDataDirectory: true,
  };
}

test("data directory requires both explicit confirmations and the canonical profile path", async () => {
  const profile = await makeDisposableProfile();
  try {
    await assert.rejects(
      canonicalizeDataDirectory({ ...profile, confirmDataDirectory: false, confirmDisposableProfile: true }),
      /confirmation/i,
    );
    await assert.rejects(
      canonicalizeDataDirectory({ ...profile, confirmDataDirectory: true, confirmDisposableProfile: false }),
      /confirmation/i,
    );
    const result = await canonicalizeDataDirectory(confirmed(profile));
    assert.equal(result.relativeDataDirectory, ".local/share/cl-go-dash");
    assert.equal(result.relativeModelsDirectory, ".ollama/models");
  } finally {
    await rm(profile.homeDirectory, { recursive: true, force: true });
  }
});

test("traversal outside the real Beaver data directory is rejected", async () => {
  const profile = await makeDisposableProfile();
  try {
    await assert.rejects(
      canonicalizeDataDirectory({
        ...confirmed(profile),
        dataDirectory: join(profile.dataDirectory, "..", "other"),
      }),
      /data directory/i,
    );
  } finally {
    await rm(profile.homeDirectory, { recursive: true, force: true });
  }
});

test("proof collection hashes model and bundle files by streaming and returns relative paths", async () => {
  const profile = await makeDisposableProfile();
  try {
    const proof = await collectNativeUpgradeProof(confirmed(profile));
    assert.equal(proof.entries.length, 2);
    assert.equal(proof.entries.every((entry) => !entry.relativePath.includes("..")), true);
    const versionEntry = proof.entries.find((entry) => entry.relativePath.endsWith("VERSION"));
    assert.equal(versionEntry.sha256, createHash("sha256").update("1.1.2\n").digest("hex"));
    assert.equal(proof.dataDirectory, ".local/share/cl-go-dash");
    assert.equal(proof.modelsDirectory, ".ollama/models");
  } finally {
    await rm(profile.homeDirectory, { recursive: true, force: true });
  }
});

test("a symlink escaping the data directory is rejected", async (t) => {
  if (process.platform === "win32") {
    t.skip("Windows junction creation requires a privileged fixture");
  }
  const profile = await makeDisposableProfile();
  const outside = await mkdtemp(join(tmpdir(), "beaver-native-proof-outside-"));
  try {
    await writeFile(join(outside, "foreign"), "outside\n");
    await symlink(outside, join(profile.dataDirectory, "ollama-bundle", "escape"));
    await assert.rejects(collectNativeUpgradeProof(confirmed(profile)), /symlink/i);
  } finally {
    await rm(profile.homeDirectory, { recursive: true, force: true });
    await rm(outside, { recursive: true, force: true });
  }
});

test("a symlink contained in an allowlisted bundle is recorded without exposing its absolute target", async (t) => {
  if (process.platform === "win32") {
    t.skip("Windows symlink creation requires a privileged fixture");
  }
  const profile = await makeDisposableProfile();
  try {
    await symlink("VERSION", join(profile.dataDirectory, "ollama-bundle", "VERSION-current"));
    const proof = await collectNativeUpgradeProof(confirmed(profile));
    const link = proof.entries.find((entry) => entry.relativePath.endsWith("VERSION-current"));
    assert.deepEqual(link, {
      relativePath: "data/ollama-bundle/VERSION-current",
      symlinkTarget: "data/ollama-bundle/VERSION",
    });
  } finally {
    await rm(profile.homeDirectory, { recursive: true, force: true });
  }
});

test("a bundle symlink cannot reach a sibling file in the Beaver data directory", async (t) => {
  if (process.platform === "win32") {
    t.skip("Windows symlink creation requires a privileged fixture");
  }
  const profile = await makeDisposableProfile();
  try {
    await writeFile(join(profile.dataDirectory, "config.json"), "{}\n");
    await symlink("../config.json", join(profile.dataDirectory, "ollama-bundle", "config-link"));
    await assert.rejects(collectNativeUpgradeProof(confirmed(profile)), /symlink/i);
  } finally {
    await rm(profile.homeDirectory, { recursive: true, force: true });
  }
});

test("sensitive files inside an allowlisted Ollama root are rejected", async () => {
  const profile = await makeDisposableProfile();
  try {
    await writeFile(join(profile.dataDirectory, "ollama-bundle", "secrets.enc"), "not a secret\n");
    await assert.rejects(collectNativeUpgradeProof(confirmed(profile)), /sensitive/i);
  } finally {
    await rm(profile.homeDirectory, { recursive: true, force: true });
  }
});

test("the published Linux bundle fits inside the bounded inventory", async () => {
  const profile = await makeDisposableProfile();
  try {
    const bundle = join(profile.dataDirectory, "ollama-bundle");
    await Promise.all(
      Array.from({ length: 1874 }, (_, index) =>
        writeFile(join(bundle, `linux-bundle-file-${index}.bin`), `${index}\n`),
      ),
    );
    const proof = await collectNativeUpgradeProof(confirmed(profile));
    assert.equal(proof.entries.length, 1876);
  } finally {
    await rm(profile.homeDirectory, { recursive: true, force: true });
  }
});

test(`collection is bounded at ${MAX_INVENTORY_ENTRIES} entries`, async () => {
  const profile = await makeDisposableProfile();
  try {
    const bundle = join(profile.dataDirectory, "ollama-bundle");
    await Promise.all(
      Array.from({ length: MAX_INVENTORY_ENTRIES + 1 }, (_, index) =>
        writeFile(join(bundle, `file-${index}.bin`), `${index}\n`),
      ),
    );
    await assert.rejects(collectNativeUpgradeProof(confirmed(profile)), /entries/i);
  } finally {
    await rm(profile.homeDirectory, { recursive: true, force: true });
  }
});

test("serialized proof is bounded at 4 MiB", () => {
  const oversized = {
    schemaVersion: 1,
    dataDirectory: ".local/share/cl-go-dash",
    modelsDirectory: ".ollama/models",
    entries: Array.from({ length: 1000 }, (_, index) => ({
      relativePath: `ollama-bundle/${index}-${"x".repeat(5000)}`,
      bytes: 1,
      sha256: "0".repeat(64),
    })),
  };
  assert.equal(MAX_PROOF_BYTES, 4 * 1024 * 1024);
  assert.throws(() => serializeNativeUpgradeProof(oversized), /size/i);
});
