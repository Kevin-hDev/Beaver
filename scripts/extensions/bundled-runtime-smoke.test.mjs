import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createHost, resetAndLoad } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostDirectory = join(root, "src-tauri", "target", "extension-host");
const executable = join(
  hostDirectory,
  "runtime",
  process.platform === "win32" ? "node.exe" : "node",
);
const fixtureDirectory = join(root, "scripts", "extensions", "fixtures", "ui", "standard-complete");

test("the prepared bundled runtime starts the extension host protocol", async () => {
  await access(executable);
  const host = createHost(join(hostDirectory, "host.mjs"), { executable });
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
  const host = createHost(join(hostDirectory, "host.mjs"), {
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
