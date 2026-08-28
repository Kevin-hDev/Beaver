import assert from "node:assert/strict";
import test from "node:test";

import {
  compareDecisionCounts,
  isProductionSource,
  scanSource,
  validateAllowlist,
} from "./check-provider-routing-boundaries.mjs";

function count(path, source) {
  return scanSource({ path, source }).length;
}

test("refuse les nouvelles décisions Rust par identifiant de route", () => {
  assert.equal(count("src-tauri/src/new.rs", 'if provider_id == "openai" {}'), 1);
  assert.equal(count("src-tauri/src/new.rs", 'match provider { "openai" => 1, _ => 0 }'), 1);
  assert.equal(count("src-tauri/src/new.rs", 'matches!(provider_id, "openai" | "xai")'), 1);
  assert.equal(count("src-tauri/src/new.rs", 'connection_id == "codex-oauth"'), 1);
});

test("refuse un prédicat de capacité nommé hors de son propriétaire", () => {
  assert.equal(count("src-tauri/src/new.rs", "if is_gpt_56(model) {}"), 1);
});

test("refuse les comparaisons TypeScript sur les identités provider", () => {
  assert.equal(count("src/new.ts", 'provider_id === "openai"'), 1);
  assert.equal(count("src/new.tsx", 'switch (route_id) { case "xai": break; }'), 1);
});

test("valide une exception documentée mais refuse une hausse silencieuse", () => {
  const allowlist = [{
    path: "src-tauri/src/owned.rs",
    owner: "route_profile",
    reason: "La définition fermée possède cette décision.",
    max_decisions: 1,
  }];
  assert.deepEqual(validateAllowlist(allowlist), allowlist);
  assert.deepEqual(compareDecisionCounts([{
    path: allowlist[0].path,
    decisions: count(allowlist[0].path, 'provider_id == "openai"'),
  }], allowlist), []);
  assert.deepEqual(compareDecisionCounts([{
    path: allowlist[0].path,
    decisions: count(allowlist[0].path, 'provider_id == "openai" || provider_id == "xai"'),
  }], allowlist), ["src-tauri/src/owned.rs: 2 > 1"]);
});

test("refuse les entrées sans propriétaire, motif ou borne valide", () => {
  for (const entry of [
    { path: "a.rs", owner: "", reason: "motif", max_decisions: 1 },
    { path: "a.rs", owner: "route", reason: "", max_decisions: 1 },
    { path: "a.rs", owner: "route", reason: "motif", max_decisions: -1 },
  ]) {
    assert.throws(() => validateAllowlist([entry]), /allowlist invalide/);
  }
});

test("exclut tests, fixtures et fichiers générés du scan de production", () => {
  for (const path of [
    "src/lib/example.test.ts",
    "src/lib/__tests__/example.ts",
    "src-tauri/src/route_tests.rs",
    "src-tauri/src/tests/example.rs",
    "src-tauri/src/fixtures/example.rs",
    "src/types/generated/provider.ts",
  ]) {
    assert.equal(isProductionSource(path), false, path);
  }
  assert.equal(isProductionSource("src-tauri/src/services/llm/route.rs"), true);
});
