import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createHost } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostScript = join(root, "src-tauri/target/extension-host/host.mjs");

test("extensions receive bounded structured core errors with retry guidance", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-extension-errors-"));
  const source = join(directory, "index.ts");
  await writeFile(
    source,
    `import { defineExtension, isBeaverExtensionError } from "@beaver/sdk";
    export default defineExtension(function (api: any) {
      api.registerTool({
        name: "errors",
        description: "Return structured core errors",
        parameters: { type: "object" },
        async execute() {
          const errors = [];
          for (const call of [() => api.info(), () => api.sessions.list()]) {
            try {
              await call();
            } catch (error) {
              errors.push({
                valid: isBeaverExtensionError(error),
                name: error.name,
                code: error.code,
                reason: error.reason,
                retryable: error.retryable
              });
            }
          }
          return JSON.stringify(errors);
        }
      });
    });`,
    { mode: 0o600 },
  );
  const host = createHost(hostScript, {
    respondToCore(message) {
      return message.method === "app.info"
        ? { error: { code: -32_000, message: "core_busy" } }
        : { error: { code: -32_601, message: "core_method_unavailable" } };
    },
  });

  try {
    await host.request("host.sync", {
      extensions: [{
        id: "com.beaver.errors",
        mainPath: source,
        manifest: { apiLevel: "stable" },
      }],
    });
    const result = await host.request("tool.call", {
      name: "com.beaver.errors.errors",
      arguments: {},
      context: { workingDirectory: directory },
    });
    const errors = JSON.parse(result.content);

    assert.deepEqual(errors, [
      {
        valid: true,
        name: "BeaverExtensionError",
        code: -32_000,
        reason: "core_busy",
        retryable: true,
      },
      {
        valid: true,
        name: "BeaverExtensionError",
        code: -32_601,
        reason: "core_method_unavailable",
        retryable: false,
      },
    ]);
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});
