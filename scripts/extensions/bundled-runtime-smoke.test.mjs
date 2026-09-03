import assert from "node:assert/strict";
import { access } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createHost } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostDirectory = join(root, "src-tauri", "target", "extension-host");
const executable = join(
  hostDirectory,
  "runtime",
  process.platform === "win32" ? "node.exe" : "node",
);

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
