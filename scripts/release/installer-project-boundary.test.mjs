import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("l'installateur reste une application Tauri minimale", async () => {
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
