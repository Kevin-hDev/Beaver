import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { resolve } from "node:path";
import { createHost } from "./host-test-client.mjs";
import {
  validateCoreApiParams,
  validateCoreApiResult,
} from "../../src-tauri/resources/extension-host/core-api-validation.mjs";

const contract = JSON.parse(await readFile(
  new URL("../../src-tauri/resources/extension-host/contract.json", import.meta.url),
  "utf8",
));

const CORE_CAPABILITIES = [
  "models",
  "memory",
  "automations",
  "subagents",
  "toolInterception",
];

const CORE_METHODS = [
  "models.list",
  "models.generate",
  "memory.list",
  "memory.read",
  "memory.write",
  "memory.archive",
  "automations.list",
  "automations.create",
  "automations.update",
  "automations.setActive",
  "automations.delete",
  "subagents.spawn",
  "subagents.list",
  "subagents.get",
  "subagents.send",
  "subagents.cancel",
];

test("les capacités optionnelles restent fermées avant un handshake valide", async () => {
  const moduleUrl = new URL(
    `../../src-tauri/resources/extension-host/extension-api-capabilities.mjs?test=${Date.now()}`,
    import.meta.url,
  );
  const capabilities = await import(moduleUrl);

  assert.deepEqual(capabilities.activeCapabilities(), contract.capabilities);
  assert.deepEqual(capabilities.negotiateCapabilities(null), contract.capabilities);
});

test("le contrat central décrit toute la seconde surface sans l'activer", () => {
  assert.deepEqual(
    contract.optionalCapabilities.slice(-CORE_CAPABILITIES.length),
    CORE_CAPABILITIES,
  );
  assert.equal(contract.methods.coreToHost.includes("tool.intercept"), true);
  assert.deepEqual(contract.events, [
    "session.turn.started",
    "session.turn.completed",
    "session.turn.failed",
    "session.turn.cancelled",
    "tool.execution.started",
    "tool.execution.finished",
    "automation.execution.started",
    "automation.execution.finished",
    "subagent.status.changed",
  ]);

  const methods = new Map(contract.methods.hostToCore.map((method) => [method.name, method]));
  assert.deepEqual(CORE_METHODS.filter((name) => !methods.has(name)), []);
  for (const name of CORE_METHODS) {
    const method = methods.get(name);
    assert.equal(method.level, "stable");
    assert.equal(method.kind, "request");
    assert.equal(CORE_CAPABILITIES.includes(method.capability), true);
    assert.equal(typeof method.requiresContext, "boolean");
    assert.equal(typeof method.idempotent, "boolean");
    assert.equal(Array.isArray(method.effects), true);
    assert.equal(method.effects.length > 0, true);
    assert.equal(method.effects.every((effect) => contract.effectClasses.includes(effect)), true);
    assert.equal(Array.isArray(method.params), true);
    assert.equal(typeof method.result, "string");
  }
});

test("les bornes et erreurs de la seconde surface ont une autorité unique", () => {
  assert.deepEqual(
    {
      maxActiveContexts: contract.limits.maxActiveContexts,
      maxContextsPerHostIdentity: contract.limits.maxContextsPerHostIdentity,
      maxCoreCallsPerHostIdentity: contract.limits.maxCoreCallsPerHostIdentity,
      maxModelGenerationsPerHostIdentity: contract.limits.maxModelGenerationsPerHostIdentity,
      maxModelGenerations: contract.limits.maxModelGenerations,
      maxModelPromptBytes: contract.limits.maxModelPromptBytes,
      maxModelOutputTokens: contract.limits.maxModelOutputTokens,
      maxModelResultBytes: contract.limits.maxModelResultBytes,
      maxEventQueuePerHost: contract.limits.maxEventQueuePerHost,
      maxEventBytes: contract.limits.maxEventBytes,
      maxInterceptors: contract.limits.maxInterceptors,
      maxAutomationsPerExtension: contract.limits.maxAutomationsPerExtension,
      maxSdkPageResults: contract.limits.maxSdkPageResults,
    },
    {
      maxActiveContexts: 64,
      maxContextsPerHostIdentity: 8,
      maxCoreCallsPerHostIdentity: 8,
      maxModelGenerationsPerHostIdentity: 2,
      maxModelGenerations: 8,
      maxModelPromptBytes: 65_536,
      maxModelOutputTokens: 4_096,
      maxModelResultBytes: 262_144,
      maxEventQueuePerHost: 32,
      maxEventBytes: 16_384,
      maxInterceptors: 8,
      maxAutomationsPerExtension: 8,
      maxSdkPageResults: 50,
    },
  );
  assert.equal(contract.timeouts.modelGenerationTimeoutMs, 25_000);
  assert.equal(contract.timeouts.interceptorHandlerTimeoutMs, 250);
  assert.equal(contract.timeouts.interceptorChainTimeoutMs, 1_000);
  assert.equal(contract.errors.retryableReasons.includes("core_request_timeout"), true);
  assert.equal(contract.errors.protocolReasons.includes("interception_individual_timeout"), true);
  assert.equal(contract.errors.protocolReasons.includes("interception_chain_timeout"), true);
});

test("le handshake n'annonce que les capacités réellement raccordées", async () => {
  const host = createHost(resolve("src-tauri/target/extension-host/host.mjs"));
  try {
    const hello = await host.request("host.hello", {
      capabilities: [...contract.capabilities, ...contract.optionalCapabilities],
    });
    assert.deepEqual(hello.capabilities, [
      ...contract.capabilities,
      "skills",
      "resources",
      "richToolResults",
      "models",
      "memory",
      "automations",
      "subagents",
      "toolInterception",
    ]);
    assert.equal(hello.capabilities.includes("models"), true);
    assert.equal(hello.capabilities.includes("toolInterception"), true);
  } finally {
    host.stop();
    await host.exited;
  }
});

test("la projection refuse méthodes, champs et budgets hors contrat", () => {
  assert.throws(() => validateCoreApiParams("unknown.method", {}), /core_method_unavailable/u);
  assert.throws(
    () => validateCoreApiParams("models.list", { unexpected: true }),
    /core_request_failed/u,
  );
  const exact = "🦫".repeat(contract.limits.maxModelPromptBytes / 4);
  assert.equal(
    validateCoreApiParams("models.generate", { prompt: exact }).prompt,
    exact,
  );
  assert.throws(
    () => validateCoreApiParams("models.generate", { prompt: `${exact}🦫` }),
    /core_request_failed/u,
  );
  const exactResult = "x".repeat(contract.limits.maxMessageBytes - 2);
  assert.equal(validateCoreApiResult(exactResult), exactResult);
  assert.throws(
    () => validateCoreApiResult(`${exactResult}x`),
    /core_request_failed/u,
  );
});
