import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createToolInterceptor } from "../../src-tauri/resources/extension-host/tool-interceptor.mjs";
import { createExtensionApi } from "../../src-tauri/resources/extension-host/extension-api.mjs";
import { negotiateCapabilities } from "../../src-tauri/resources/extension-host/extension-api-capabilities.mjs";
import { OPTIONAL_CAPABILITIES } from "../../src-tauri/resources/extension-host/contract.mjs";
import {
  callExtensionInterceptor,
  loadExtensionWithApi,
  resetExtensions,
} from "../../src-tauri/resources/extension-host/loader.mjs";

negotiateCapabilities(OPTIONAL_CAPABILITIES);

test("late_or_mutating_response_cannot_execute_tool", async () => {
  const interceptor = createToolInterceptor(true);
  let reads = 0;
  interceptor.register(() => ({
    get decision() {
      reads += 1;
      return reads === 1 ? "deny" : "continue";
    },
  }));

  const result = await interceptor.invoke({
    toolName: "write_file",
    effect: "local-write",
    mode: "auto",
  });

  assert.equal(result.decision, "deny");
  assert.equal(reads, 1);
  assert.ok(Object.isFrozen(result));
});

test("one bounded interceptor is registered per extension", async () => {
  const interceptor = createToolInterceptor(true);
  const cleanup = interceptor.register(() => ({ decision: "continue" }));
  assert.equal(interceptor.contributions().length, 1);
  assert.throws(() => interceptor.register(() => ({ decision: "continue" })), /invalid_interceptor/);
  assert.equal((await interceptor.invoke({ toolName: "read_file", effect: "read-only", mode: "auto" })).decision, "continue");
  cleanup();
  assert.equal(interceptor.contributions().length, 0);
  assert.equal((await interceptor.invoke({ toolName: "read_file", effect: "read-only", mode: "auto" })).decision, "invalid");
});

test("handler failures and invalid decisions fail closed", async () => {
  const failed = createToolInterceptor(true);
  failed.register(() => { throw new Error("private detail"); });
  assert.equal((await failed.invoke({ toolName: "bash", effect: "process", mode: "auto" })).decision, "failed");

  const invalid = createToolInterceptor(true);
  invalid.register(() => ({ decision: "allow" }));
  assert.equal((await invalid.invoke({ toolName: "bash", effect: "process", mode: "auto" })).decision, "invalid");
});

test("loaded extensions publish and execute their single interceptor", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-interceptor-"));
  const mainPath = join(directory, "index.mjs");
  await writeFile(mainPath, `export default function (api) {
    api.interceptTool((call) => ({ decision: call.toolName === "bash" ? "deny" : "continue" }));
  }`);
  try {
    const loaded = await loadExtensionWithApi({
      id: "test.interceptor",
      mainPath,
      manifest: { apiLevel: "stable" },
    }, createExtensionApi);
    assert.equal(loaded.error, undefined);
    assert.equal(loaded.contributions.interceptors.length, 1);
    assert.equal((await callExtensionInterceptor("test.interceptor", {
      toolName: "bash",
      effect: "process",
      mode: "auto",
    })).decision, "deny");
  } finally {
    await resetExtensions();
    await rm(directory, { recursive: true, force: true });
  }
});
