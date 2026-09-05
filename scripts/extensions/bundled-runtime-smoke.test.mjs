import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createHost, resetAndLoad } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostDirectory = join(root, "src-tauri", "target", "extension-host");
const executable = join(
  hostDirectory,
  "runtime",
  process.platform === "win32" ? "node.exe" : "node",
);
const fixtureDirectory = join(root, "scripts", "extensions", "fixtures", "ui", "standard-complete");
const themeFixtureDirectory = join(root, "scripts", "extensions", "fixtures", "ui", "theme-valid");
const catalogPath = join(hostDirectory, "builtin-plugins", "catalog.json");
const hostScript = join(hostDirectory, "host.mjs");
const { TIMEOUTS } = await import(pathToFileURL(join(hostDirectory, "contract.mjs")));

test("the prepared bundled runtime starts the extension host protocol", async () => {
  await access(executable);
  const host = createHost(hostScript, { executable });
  try {
    const hello = await host.request("host.hello", {});
    assert.equal(hello.apiVersion, "1");
    assert.equal(typeof hello.nodeVersion, "string");
    assert.ok(hello.nodeVersion.length > 0);
  } finally {
    host.stop();
    await host.exited;
  }
});

test("the prepared bundled runtime loads a real UI extension", async () => {
  await access(executable);
  const manifest = JSON.parse(
    await readFile(join(fixtureDirectory, "beaver-extension.json"), "utf8"),
  );
  const stages = [];
  const host = createHost(hostScript, {
    executable,
    onNotification(message) {
      if (stages.length < 3 && typeof message.params?.stage === "string") {
        stages.push(message.params.stage);
      }
    },
  });
  try {
    const result = await resetAndLoad(host, [{
      id: manifest.id,
      mainPath: join(fixtureDirectory, manifest.main),
      manifest,
    }]);
    assert.deepEqual(stages, ["import", "activate", "register"]);
    assert.equal(result.extensions[0]?.id, manifest.id);
    assert.equal(result.extensions[0]?.contributions?.ui?.length, 4);
  } catch (error) {
    throw new Error(`Real extension load failed after stages: ${stages.join(",") || "none"}`, {
      cause: error,
    });
  } finally {
    host.stop();
    await host.exited;
  }
});

test("a real extension loads while the official suite remains resident", async () => {
  await access(executable);
  const catalog = JSON.parse(await readFile(catalogPath, "utf8"));
  const manifest = JSON.parse(
    await readFile(join(fixtureDirectory, "beaver-extension.json"), "utf8"),
  );
  const officialHost = createHost(hostScript, {
    executable,
    requestTimeoutMs: TIMEOUTS.hostRequestTimeoutMs,
  });
  const thirdPartyHost = createHost(hostScript, {
    executable,
    requestTimeoutMs: TIMEOUTS.hostRequestTimeoutMs,
  });
  try {
    const officialExtensions = catalog.plugins.map(({ manifest: officialManifest }) => ({
      id: officialManifest.id,
      mainPath: join(hostDirectory, officialManifest.main),
      manifest: officialManifest,
    }));
    const official = await resetAndLoad(officialHost, officialExtensions);
    assert.equal(official.extensions.length, 4);
    assert.ok(official.extensions.every((extension) => !extension.error));

    const thirdParty = await resetAndLoad(thirdPartyHost, [{
      id: manifest.id,
      mainPath: join(fixtureDirectory, manifest.main),
      manifest,
    }]);
    assert.equal(thirdParty.extensions[0]?.id, manifest.id);
    assert.equal(thirdParty.extensions[0]?.contributions?.ui?.length, 4);
  } finally {
    officialHost.stop();
    thirdPartyHost.stop();
    await Promise.all([officialHost.exited, thirdPartyHost.exited]);
  }
});

test("a growing extension set loads after every host generation is replaced", async () => {
  await access(executable);
  const catalog = JSON.parse(await readFile(catalogPath, "utf8"));
  const standardManifest = JSON.parse(
    await readFile(join(fixtureDirectory, "beaver-extension.json"), "utf8"),
  );
  const themeManifest = JSON.parse(
    await readFile(join(themeFixtureDirectory, "beaver-extension.json"), "utf8"),
  );
  const officialExtensions = catalog.plugins.map(({ manifest }) => ({
    id: manifest.id,
    mainPath: join(hostDirectory, manifest.main),
    manifest,
  }));

  const firstOfficial = createHost(hostScript, { executable });
  const firstStandard = createHost(hostScript, { executable });
  try {
    await resetAndLoad(firstOfficial, officialExtensions);
    await resetAndLoad(firstStandard, [{
      id: standardManifest.id,
      mainPath: join(fixtureDirectory, standardManifest.main),
      manifest: standardManifest,
    }]);
  } finally {
    firstOfficial.stop();
    firstStandard.stop();
    await Promise.all([firstOfficial.exited, firstStandard.exited]);
  }

  const secondOfficial = createHost(hostScript, { executable });
  const secondStandard = createHost(hostScript, { executable });
  const secondTheme = createHost(hostScript, { executable });
  try {
    const [official, standard, theme] = await Promise.all([
      resetAndLoad(secondOfficial, officialExtensions),
      resetAndLoad(secondStandard, [{
        id: standardManifest.id,
        mainPath: join(fixtureDirectory, standardManifest.main),
        manifest: standardManifest,
      }]),
      resetAndLoad(secondTheme, [{
        id: themeManifest.id,
        mainPath: join(themeFixtureDirectory, themeManifest.main),
        manifest: themeManifest,
      }]),
    ]);
    assert.ok(official.extensions.every((extension) => !extension.error));
    assert.equal(standard.extensions[0]?.id, standardManifest.id);
    assert.equal(theme.extensions[0]?.id, themeManifest.id);
  } finally {
    secondOfficial.stop();
    secondStandard.stop();
    secondTheme.stop();
    await Promise.all([
      secondOfficial.exited,
      secondStandard.exited,
      secondTheme.exited,
    ]);
  }
});
