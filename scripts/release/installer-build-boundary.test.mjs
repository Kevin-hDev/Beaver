import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { access, readFile } from "node:fs/promises";
import test from "node:test";

const SHARED_BEAVER_ASSET = "src/assets/QPXAq01-anime.svg";

test("le backend de l'installateur reste une application Tauri minimale", async () => {
  const cargo = await readFile("installer/src-tauri/Cargo.toml", "utf8");

  assert.match(cargo, /^name = "beaver-installer"$/mu);
  for (const forbidden of [
    "cl-go-dash",
    "cef",
    "git2",
    "keyring",
    "ollama_manager",
  ]) {
    assert.doesNotMatch(cargo, new RegExp(forbidden, "u"));
  }
});

test("la configuration active uniquement la capability minimale", async () => {
  const config = JSON.parse(
    await readFile("installer/src-tauri/tauri.conf.json", "utf8"),
  );
  const capability = JSON.parse(
    await readFile("installer/src-tauri/capabilities/default.json", "utf8"),
  );

  assert.deepEqual(config.app.security.capabilities, ["default"]);
  assert.equal(capability.identifier, "default");
  assert.deepEqual(capability.windows, ["main"]);
  assert.deepEqual(capability.permissions, ["core:window:allow-close"]);
});

test("le dessin partagé du castor appartient au dépôt", async () => {
  await access(SHARED_BEAVER_ASSET);
  const ignored = spawnSync("git", ["check-ignore", "-q", SHARED_BEAVER_ASSET]);

  assert.equal(ignored.status, 1);
});
