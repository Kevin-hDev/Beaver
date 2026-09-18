import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const CHECKER = fileURLToPath(new URL("../check-react-component-calls.mjs", import.meta.url));

for (const scenario of [
  { file: "panel.ts", imported: "@/components/settings/settings", status: 1 },
  { file: "panel.ts", imported: "@/components/ui/button", status: 0 },
  { file: "__tests__/panel.ts", imported: "@/components/settings/settings", status: 0 },
]) {
  test(`extension UI boundary: ${scenario.file} importing ${scenario.imported}`, async () => {
    const root = await mkdtemp(join(tmpdir(), "beaver-react-boundaries-"));
    try {
      const target = join(root, "src/features/extension-ui", scenario.file);
      await mkdir(dirname(target), { recursive: true });
      await writeFile(target, `import { Widget } from "${scenario.imported}";\n`);
      const result = spawnSync(process.execPath, [CHECKER], {
        cwd: root, encoding: "utf8", timeout: 30_000, maxBuffer: 65_536, windowsHide: true,
      });
      assert.ifError(result.error);
      assert.equal(result.status, scenario.status, result.stderr);
      if (scenario.status !== 0) assert.match(result.stderr, /only shared UI primitives/u);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
}
