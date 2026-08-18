import assert from "node:assert/strict";
import { resolve } from "node:path";
import test from "node:test";

import { loadConfigFromFile } from "vite";

test("Vite ignore les deux répertoires de compilation Cargo", async () => {
  const loaded = await loadConfigFromFile(
    { command: "serve", mode: "test" },
    resolve("vite.config.ts"),
  );
  assert.ok(loaded);

  const ignored = loaded.config.server?.watch?.ignored;
  assert.equal(typeof ignored, "function");
  assert.equal(
    ignored(resolve("src-tauri", "target", "debug", "deps", "cl_go_dash_lib.dll")),
    true,
  );
  assert.equal(
    ignored(resolve("target", "debug", "deps", "cl_go_dash_lib.dll")),
    true,
  );
  assert.equal(ignored(resolve("src", "App.tsx")), false);
});
