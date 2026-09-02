import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createHost, resetAndLoad } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostScript = join(root, "src-tauri/target/extension-host/host.mjs");

test("reports safe structured diagnostics for syntax errors", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-extension-diagnostic-"));
  const source = join(directory, "broken.ts");
  await writeFile(source, "export default function ( {", { mode: 0o600 });
  const host = createHost(hostScript);
  try {
    const sync = await resetAndLoad(host, [{
        id: "com.beaver.broken",
        mainPath: source,
        manifest: { apiLevel: "stable" },
      }]);

    assert.equal(sync.extensions[0].error, "load_failed");
    assert.equal(sync.extensions[0].diagnostic.stage, "import");
    assert.equal(sync.extensions[0].diagnostic.code, "syntax_error");
    assert.equal(sync.extensions[0].diagnostic.file, "broken.ts");
  } finally {
    host.stop();
    await rm(directory, { recursive: true, force: true });
  }
});
