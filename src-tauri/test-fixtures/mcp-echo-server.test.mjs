import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

const fixture = fileURLToPath(new URL("./mcp-echo-server.mjs", import.meta.url));

function run(request) {
  return spawnSync(process.execPath, [fixture], {
    input: `${JSON.stringify(request)}\n`,
    encoding: "utf8",
    timeout: 3000,
    maxBuffer: 64 * 1024,
    shell: false,
  });
}

test("une méthode inconnue n'est jamais un succès vide", () => {
  const response = run({ jsonrpc: "2.0", id: 1, method: "unknown/method" });
  assert.equal(response.status, 0);
  assert.equal(JSON.parse(response.stdout).error.code, -32601);
});

test("l'initialisation annonce la version et les outils réels", () => {
  const response = run({
    jsonrpc: "2.0", id: 2, method: "initialize",
    params: { protocolVersion: "2025-03-26", capabilities: {}, clientInfo: { name: "test", version: "1" } },
  });
  assert.equal(response.status, 0);
  const result = JSON.parse(response.stdout).result;
  assert.equal(result.protocolVersion, "2025-03-26");
  assert.deepEqual(result.capabilities, { tools: {} });
  assert.equal(result.serverInfo.name, "beaver-test-echo");
});

test("une version non prise en charge est refusée", () => {
  const response = run({
    jsonrpc: "2.0", id: 3, method: "initialize",
    params: { protocolVersion: "2026-07-28" },
  });
  assert.equal(JSON.parse(response.stdout).error.code, -32602);
});

test("la découverte absente et un outil inconnu sont refusés", () => {
  const discovery = run({ jsonrpc: "2.0", id: 4, method: "server/discover" });
  const tool = run({ jsonrpc: "2.0", id: 5, method: "tools/call", params: { name: "other" } });
  assert.equal(JSON.parse(discovery.stdout).error.code, -32601);
  assert.equal(JSON.parse(tool.stdout).error.code, -32602);
});

test("les notifications ne reçoivent aucune réponse", () => {
  const response = run({ jsonrpc: "2.0", method: "notifications/initialized" });
  assert.equal(response.status, 0);
  assert.equal(response.stdout, "");
});

test("une ligne JSON invalide arrête la fixture", () => {
  const response = spawnSync(process.execPath, [fixture], {
    input: "{invalid\n", encoding: "utf8", timeout: 3000,
    maxBuffer: 64 * 1024, shell: false,
  });
  assert.equal(response.status, 1);
  assert.equal(response.stdout, "");
});
