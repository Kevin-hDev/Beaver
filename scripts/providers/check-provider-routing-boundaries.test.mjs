import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  compareDecisionCounts,
  isProductionSource,
  scanSource,
  validateAllowlist,
} from "./check-provider-routing-boundaries.mjs";

function count(path, source) {
  return scanSource({ path, source }).length;
}

test("le scan réel compare les chemins natifs à l'autorité portable", () => {
  const root = fs.mkdtempSync(path.join(tmpdir(), "beaver-routing-"));
  const checker = fileURLToPath(new URL("./check-provider-routing-boundaries.mjs", import.meta.url));
  try {
    fs.mkdirSync(path.join(root, "src"));
    fs.mkdirSync(path.join(root, "scripts/providers"), { recursive: true });
    fs.writeFileSync(path.join(root, "src/owned.ts"), 'providerId === "openai";');
    fs.writeFileSync(path.join(root, "scripts/providers/provider-branch-allowlist.json"), JSON.stringify([{
      path: "src/owned.ts",
      owner: "route_profile",
      reason: "La fixture possède une décision de routage.",
      max_decisions: 1,
    }]));
    const result = spawnSync(process.execPath, [checker], {
      cwd: root, encoding: "utf8", timeout: 30_000, maxBuffer: 64 * 1024, windowsHide: true,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /Provider routing boundaries: 1 fichiers autorisés\./u);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("refuse les nouvelles décisions Rust par identifiant de route", () => {
  assert.equal(count("src-tauri/src/new.rs", 'if provider_id == "openai" {}'), 1);
  assert.equal(count("src-tauri/src/new.rs", 'match provider { "openai" => 1, _ => 0 }'), 1);
  assert.equal(count("src-tauri/src/new.rs", 'matches!(provider_id, "openai" | "xai")'), 1);
  assert.equal(count("src-tauri/src/new.rs", 'connection_id == "codex-oauth"'), 1);
});

test("continue le scan après un item cfg test sans compter un module de tests", () => {
  const source = `
#[cfg(test)]
pub fn parse_fixture() {}

#[cfg(test)]
mod tests {
  fn ignored() {
    let _json = r#"{"provider_id":"inside-test"}"#;
    if provider_id == "inside-test" {}
  }
}

pub fn production() { if provider_id == "openai" {} }
`;

  assert.equal(count("src-tauri/src/new.rs", source), 1);
});

test("refuse les décisions portées par les identités de route du chat", () => {
  assert.equal(count("src-tauri/src/new.rs", 'route.chat_provider_id == "xai-oauth"'), 1);
  assert.equal(count("src/new.ts", 'route.chatProviderId === "xai-oauth"'), 1);
});

test("refuse les comparaisons Rust avec une constante provider nommée", () => {
  assert.equal(
    count(
      "src-tauri/src/new.rs",
      "provider_id == crate::services::codex_client::PROVIDER_ID",
    ),
    1,
  );
  assert.equal(count("src-tauri/src/new.rs", "PROVIDER_ID != route_id"), 1);
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
  assert.deepEqual(compareDecisionCounts([], allowlist), [
    "src-tauri/src/owned.rs: autorisation périmée",
  ]);
  assert.deepEqual(compareDecisionCounts([{
    path: allowlist[0].path,
    decisions: 0,
  }], allowlist), ["src-tauri/src/owned.rs: borne obsolète 1, valeur 0"]);
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

test("les consommateurs frontend génériques ne décident plus selon le provider", () => {
  const files = [
    "src/components/providers/usage/provider-usage-links.ts",
    "src/hooks/oauth-models.ts",
    "src/hooks/use-context-progress.ts",
    "src/hooks/use-context-usage.ts",
  ];
  for (const path of files) {
    const source = fs.existsSync(path) ? fs.readFileSync(path, "utf8") : "";
    assert.equal(count(path, source), 0, path);
  }
});

test("le sélecteur porte une seule exception de présentation Anthropic", () => {
  const path = "src/lib/reasoning-modes.ts";
  const source = fs.readFileSync(path, "utf8");
  assert.equal(count(path, source), 1);
});
