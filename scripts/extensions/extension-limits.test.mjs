import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { LIMITS } from "../../src-tauri/resources/extension-host/contract.mjs";
import { OFFICE_LIMITS } from "../../src-tauri/resources/extension-host/builtin-plugins/common/constants.mjs";
import { createHost } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostScript = join(root, "src-tauri/target/extension-host/host.mjs");

test("isolates only the extension that exceeds the shared global tool limit", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-extension-limits-"));
  const source = join(directory, "index.ts");
  await writeFile(
    source,
    `export default function (api: any) {
      for (let index = 0; index < ${LIMITS.maxToolsPerExtension}; index += 1) {
        api.registerTool({
          name: "tool-" + index,
          description: "Bounded test tool",
          parameters: { type: "object" },
          execute() { return "ok"; }
        });
      }
    }`,
    { mode: 0o600 },
  );
  const acceptedCount = Math.floor(
    LIMITS.maxTools / LIMITS.maxToolsPerExtension,
  );
  const specifications = Array.from(
    { length: acceptedCount + 1 },
    (_, index) => ({
      id: `com.beaver.limit-${index}`,
      mainPath: source,
      manifest: { apiLevel: "stable" },
    }),
  );
  const host = createHost(hostScript);

  try {
    const sync = await host.request("host.sync", { extensions: specifications });
    const active = sync.extensions.filter((extension) => !extension.error);
    const rejected = sync.extensions.filter((extension) => extension.error);

    assert.equal(active.length, acceptedCount);
    assert.equal(rejected.length, 1);
    assert.equal(rejected[0].id, `com.beaver.limit-${acceptedCount}`);
    assert.equal(rejected[0].diagnostic.code, "registration_failed");
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});

test("shares and enforces one working-directory limit across the host", async () => {
  assert.equal(
    OFFICE_LIMITS.maxPathChars,
    LIMITS.maxWorkingDirectoryChars,
  );
  assert.ok(OFFICE_LIMITS.maxPreviewBytes < OFFICE_LIMITS.maxStructuredResultBytes);
  assert.ok(OFFICE_LIMITS.maxStructuredResultBytes < LIMITS.maxMessageBytes);
  const directory = await mkdtemp(join(tmpdir(), "beaver-context-limit-"));
  const source = join(directory, "index.ts");
  await writeFile(
    source,
    `export default function (api: any) {
      api.registerTool({
        name: "context",
        description: "Read the context",
        parameters: { type: "object" },
        execute(_args: unknown, context: any) { return context.workingDirectory; }
      });
    }`,
    { mode: 0o600 },
  );
  const host = createHost(hostScript);
  try {
    await host.request("host.sync", {
      extensions: [{
        id: "com.beaver.context-limit",
        mainPath: source,
        manifest: { apiLevel: "stable" },
      }],
    });
    await assert.rejects(
      host.request("tool.call", {
        name: "com.beaver.context-limit.context",
        arguments: {},
        context: { workingDirectory: "x".repeat(LIMITS.maxWorkingDirectoryChars + 1) },
      }),
    );
    assert.equal((await host.request("host.hello", {})).apiVersion, "1");
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});
