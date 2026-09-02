import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createHost, resetAndLoad } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostScript = join(root, "src-tauri/target/extension-host/host.mjs");

test("rejects oversized protocol identifiers before tracking them", async () => {
  const child = spawn(process.execPath, [hostScript], {
    shell: false,
    stdio: ["pipe", "ignore", "ignore"],
  });
  const exited = once(child, "exit");
  child.stdin.end(`${JSON.stringify({
    jsonrpc: "2.0",
    id: "x".repeat(129),
    method: "host.hello",
    params: {},
  })}\n`);

  const [code] = await exited;
  assert.equal(code, 1);
});

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
    await resetAndLoad(host, [{
        id: "com.beaver.errors",
        mainPath: source,
        manifest: { apiLevel: "stable" },
      }]);
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

test("enforces the generated method level for stable and advanced calls", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-extension-levels-"));
  const source = join(directory, "index.ts");
  await writeFile(
    source,
    `export default function (api: any) {
      api.registerTool({
        name: "levels",
        description: "Check generated method levels",
        parameters: { type: "object" },
        async execute() {
          const stable = await api.call("app.info");
          let advancedRejected = false;
          try { await api.unstable.call("app.info"); }
          catch (error) { advancedRejected = error.message === "core_method_unavailable"; }
          let notificationRejected = false;
          try { await api.call("host.load.stage", { stage: "import" }); }
          catch (error) { notificationRejected = error.message === "core_method_unavailable"; }
          return JSON.stringify({ stable, advancedRejected, notificationRejected });
        }
      });
    }`,
    { mode: 0o600 },
  );
  let forwardedInternalNotification = 0;
  const host = createHost(hostScript, {
    respondToCore(message) {
      if (message.method === "host.load.stage") {
        forwardedInternalNotification += 1;
        return { result: "unexpected" };
      }
      return message.method === "app.info"
        ? { result: { apiVersion: "1" } }
        : { error: { code: -32_601, message: "core_method_unavailable" } };
    },
  });
  try {
    await resetAndLoad(host, [{
      id: "com.beaver.levels",
      mainPath: source,
      manifest: { apiLevel: "advanced" },
    }]);
    const result = await host.request("tool.call", {
      name: "com.beaver.levels.levels",
      arguments: {},
      context: { workingDirectory: directory },
    });
    assert.deepEqual(JSON.parse(result.content), {
      stable: { apiVersion: "1" },
      advancedRejected: true,
      notificationRejected: true,
    });
    assert.equal(forwardedInternalNotification, 0);
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});
