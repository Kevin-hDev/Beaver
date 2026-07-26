import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import * as cefExtract from "./cef-extract.mjs";

test("Windows CEF layout is flattened for cef-dll-sys", async (context) => {
  assert.equal(typeof cefExtract.normalizeWindowsCefLayout, "function");
  if (typeof cefExtract.normalizeWindowsCefLayout !== "function") return;

  const root = await mkdtemp(join(tmpdir(), "clgo-cef-layout-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(join(root, "Release"));
  await mkdir(join(root, "Resources", "locales"), { recursive: true });
  await writeFile(join(root, "Release", "libcef.dll"), "runtime");
  await writeFile(join(root, "Resources", "resources.pak"), "resources");
  await writeFile(join(root, "Resources", "locales", "en-US.pak"), "locale");
  await writeFile(join(root, "CMakeLists.txt"), "cmake");

  await cefExtract.normalizeWindowsCefLayout(root);

  await assert.rejects(() => rm(join(root, "Release")));
  await assert.rejects(() => rm(join(root, "Resources")));
  assert.equal(await read(join(root, "libcef.dll")), "runtime");
  assert.equal(await read(join(root, "resources.pak")), "resources");
  assert.equal(await read(join(root, "locales", "en-US.pak")), "locale");
  assert.equal(await read(join(root, "CMakeLists.txt")), "cmake");
});

async function read(path) {
  return import("node:fs/promises").then(({ readFile }) => readFile(path, "utf8"));
}

test("an old Windows CEF layout marker is invalidated", async (context) => {
  assert.equal(typeof cefExtract.isCurrentCefLayout, "function");
  if (typeof cefExtract.isCurrentCefLayout !== "function") return;

  const current = await mkdtemp(join(tmpdir(), "clgo-cef-current-"));
  context.after(() => rm(current, { recursive: true, force: true }));
  await writeFile(join(current, ".clgo-sha256"), "pinned-hash\n");
  await writeFile(join(current, "CMakeLists.txt"), "cmake");

  assert.equal(
    await cefExtract.isCurrentCefLayout(
      current,
      { sha256: "pinned-hash" },
      "win32",
    ),
    false,
  );
});
