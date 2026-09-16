import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { checkPackagedVoiceCatalog } from "./check-packaged-resources.mjs";

async function fixture() {
  const root = await mkdtemp(join(tmpdir(), "beaver-voice-resource-"));
  const resources = join(root, "bundle");
  const source = join(root, "voice-catalog.json");
  const bundledDirectory = join(resources, "resources");
  await mkdir(bundledDirectory, { recursive: true });
  await writeFile(source, '{"version":1}\n');
  return { root, resources, source, bundled: join(bundledDirectory, "voice-catalog.json") };
}

test("accepts the byte-identical packaged catalogue", async (context) => {
  const paths = await fixture();
  context.after(() => rm(paths.root, { recursive: true, force: true }));
  await writeFile(paths.bundled, '{"version":1}\n');
  await checkPackagedVoiceCatalog(paths.resources, paths.source);
});

test("rejects a missing packaged catalogue", async (context) => {
  const paths = await fixture();
  context.after(() => rm(paths.root, { recursive: true, force: true }));
  await assert.rejects(checkPackagedVoiceCatalog(paths.resources, paths.source));
});

test("rejects a changed packaged catalogue", async (context) => {
  const paths = await fixture();
  context.after(() => rm(paths.root, { recursive: true, force: true }));
  await writeFile(paths.bundled, '{"version":2}\n');
  await assert.rejects(checkPackagedVoiceCatalog(paths.resources, paths.source));
});

test("rejects a packaged catalogue reached through a symbolic link", async (context) => {
  const paths = await fixture();
  context.after(() => rm(paths.root, { recursive: true, force: true }));
  await symlink(paths.source, paths.bundled);
  await assert.rejects(checkPackagedVoiceCatalog(paths.resources, paths.source));
});
