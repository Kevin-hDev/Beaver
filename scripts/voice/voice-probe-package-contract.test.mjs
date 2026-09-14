import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const tauriConfigPath = new URL("../../src-tauri/tauri.conf.json", import.meta.url);
const cargoPath = new URL("../../src-tauri/Cargo.toml", import.meta.url);
const entitlementsPath = new URL("../../src-tauri/Entitlements.plist", import.meta.url);
const infoPlistPath = new URL("../../src-tauri/Info.plist", import.meta.url);

async function readOptional(path) {
  return readFile(path, "utf8").catch((error) => {
    if (error?.code === "ENOENT") return null;
    throw error;
  });
}

test("the voice probe is packaged only for macOS and Windows", async () => {
  const [cargo, entitlements, infoPlist] = await Promise.all([
    readFile(cargoPath, "utf8"),
    readFile(entitlementsPath, "utf8"),
    readOptional(infoPlistPath),
  ]);
  const tauriConfig = JSON.parse(await readFile(tauriConfigPath, "utf8"));

  assert.notEqual(infoPlist, null, "src-tauri/Info.plist must exist");
  assert.equal(tauriConfig.bundle.macOS.infoPlist, "Info.plist");
  assert.match(entitlements, /com\.apple\.security\.device\.audio-input/);
  assert.match(infoPlist, /NSMicrophoneUsageDescription/);
  assert.match(
    cargo,
    /^voice-probe = \["dep:block2", "dep:cpal", "dep:objc2-av-foundation"\]$/m,
  );
  assert.match(cargo, /^objc2-av-foundation = .*optional = true.*$/m);
  assert.match(cargo, /^\[target\.'cfg\(any\(target_os = "macos", windows\)\)'\.dependencies\]$/m);
  assert.match(cargo, /^cpal = \{ version = "=0\.18\.2", optional = true \}$/m);
});
