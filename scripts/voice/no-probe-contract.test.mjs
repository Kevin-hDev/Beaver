import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import test from "node:test";

test("the standard product exposes no temporary microphone probe", async () => {
  const [cargo, handler, features] = await Promise.all([
    readFile("src-tauri/Cargo.toml", "utf8"),
    readFile("src-tauri/src/invoke_handler.rs", "utf8"),
    readdir("src/features"),
  ]);
  assert.doesNotMatch(cargo, /voice-probe/);
  assert.doesNotMatch(handler, /voice_probe_(?:available|start|stop)/);
  assert.ok(!features.includes("voice-probe"));
});
