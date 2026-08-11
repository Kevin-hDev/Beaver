import assert from "node:assert/strict";
import { link, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  BUNDLE_MARKERS,
  patchTauriModuleMarker,
  prepareTauriBootstrapMarker,
  verifyTauriBundleMarkers,
} from "./tauri-bundle-marker.mjs";

async function withTemporaryDirectory(callback) {
  const directory = await realpath(
    await mkdtemp(join(tmpdir(), "beaver-bundle-marker-")),
  );
  try {
    await callback(directory);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

test("bootstrap preparation adds exactly one official unknown marker", async () => {
  await withTemporaryDirectory(async (directory) => {
    const bootstrap = join(directory, "cl-go-dash.exe");
    await writeFile(bootstrap, Buffer.from("MZ\0bootstrap", "ascii"));

    await prepareTauriBootstrapMarker(bootstrap);

    const result = await readFile(bootstrap);
    assert.equal(result.subarray(-BUNDLE_MARKERS.unknown.length).toString("ascii"), BUNDLE_MARKERS.unknown);
    await assert.rejects(() => prepareTauriBootstrapMarker(bootstrap), /validation failed/u);
  });
});

test("module patching replaces the unique unknown marker with the real type", async () => {
  await withTemporaryDirectory(async (directory) => {
    const module = join(directory, "cl-go-dash.dll");
    await writeFile(module, Buffer.from(`MZ-prefix-${BUNDLE_MARKERS.unknown}-suffix`, "ascii"));

    await patchTauriModuleMarker(module, "nsis");

    const result = (await readFile(module)).toString("ascii");
    assert.equal(result.includes(BUNDLE_MARKERS.unknown), false);
    assert.equal(result.includes(BUNDLE_MARKERS.nsis), true);
  });
});

test("module patching fails closed on missing, repeated or unknown markers", async () => {
  await withTemporaryDirectory(async (directory) => {
    for (const [name, body] of [
      ["missing.dll", "MZ-no-marker"],
      ["repeated.dll", `${BUNDLE_MARKERS.unknown}${BUNDLE_MARKERS.unknown}`],
    ]) {
      const module = join(directory, name);
      await writeFile(module, Buffer.from(body, "ascii"));
      await assert.rejects(() => patchTauriModuleMarker(module, "nsis"), /validation failed/u);
    }
    const valid = join(directory, "valid.dll");
    await writeFile(valid, Buffer.from(BUNDLE_MARKERS.unknown, "ascii"));
    await assert.rejects(() => patchTauriModuleMarker(valid, "zip"), /validation failed/u);
  });
});

test("module patching refuses a hard-linked destination without modifying its source", async () => {
  await withTemporaryDirectory(async (directory) => {
    const source = join(directory, "source.dll");
    const destination = join(directory, "destination.dll");
    const original = Buffer.from(`MZ-${BUNDLE_MARKERS.unknown}`, "ascii");
    await writeFile(source, original);
    await link(source, destination);

    await assert.rejects(
      () => patchTauriModuleMarker(destination, "nsis"),
      /validation failed/u,
    );
    assert.deepEqual(await readFile(source), original);
  });
});

test("package verification requires the same real type in bootstrap and module", async () => {
  await withTemporaryDirectory(async (directory) => {
    const bootstrap = join(directory, "cl-go-dash.exe");
    const module = join(directory, "cl-go-dash.dll");
    const referenceBootstrap = join(directory, "reference.exe");
    const referenceModule = join(directory, "reference.dll");
    await writeFile(referenceBootstrap, Buffer.from(`MZ-${BUNDLE_MARKERS.unknown}`, "ascii"));
    await writeFile(
      referenceModule,
      Buffer.from(`MZ-${BUNDLE_MARKERS.nsis}-${BUNDLE_MARKERS.unknown}-tail`, "ascii"),
    );
    await writeFile(bootstrap, Buffer.from(`MZ-${BUNDLE_MARKERS.nsis}`, "ascii"));
    await writeFile(
      module,
      Buffer.from(`MZ-${BUNDLE_MARKERS.nsis}-${BUNDLE_MARKERS.nsis}-tail`, "ascii"),
    );

    await verifyTauriBundleMarkers({
      bootstrap,
      bundleType: "nsis",
      module,
      referenceBootstrap,
      referenceModule,
    });
    await writeFile(
      module,
      Buffer.from(`MZ-${BUNDLE_MARKERS.nsis}-${BUNDLE_MARKERS.msi}-tail`, "ascii"),
    );
    await assert.rejects(
      () =>
        verifyTauriBundleMarkers({
          bootstrap,
          bundleType: "nsis",
          module,
          referenceBootstrap,
          referenceModule,
        }),
      /validation failed/u,
    );
  });
});
