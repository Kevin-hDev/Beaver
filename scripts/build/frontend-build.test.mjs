import assert from "node:assert/strict";
import { mkdtemp, mkdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { buildFrontend } from "./frontend-build.mjs";

async function createRepository() {
  const root = await mkdtemp(join(tmpdir(), "beaver-frontend-build-"));
  const files = [
    "scripts/check-react-component-calls.mjs",
    "node_modules/typescript/bin/tsc",
    "node_modules/vite/bin/vite.js",
  ];
  for (const file of files) {
    const path = join(root, file);
    await mkdir(join(path, ".."), { recursive: true });
    await writeFile(path, "");
  }
  return realpath(root);
}

test("exécute les trois étapes frontend avec Node et sans npm récursif", async () => {
  const root = await createRepository();
  const calls = [];
  try {
    await buildFrontend({ repoRoot: root, run: async (spec) => calls.push(spec) });
    assert.deepEqual(calls, [
      {
        command: process.execPath,
        args: [join(root, "scripts/check-react-component-calls.mjs")],
        cwd: root,
      },
      {
        command: process.execPath,
        args: [join(root, "node_modules/typescript/bin/tsc")],
        cwd: root,
      },
      {
        command: process.execPath,
        args: [join(root, "node_modules/vite/bin/vite.js"), "build"],
        cwd: root,
      },
    ]);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("refuse un dépôt non canonique ou incomplet", async () => {
  await assert.rejects(
    () => buildFrontend({ repoRoot: join(process.cwd(), ".."), run: async () => {} }),
    /Frontend build failed/,
  );
});
