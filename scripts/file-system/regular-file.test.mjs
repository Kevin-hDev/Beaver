import assert from "node:assert/strict";
import { renameSync, symlinkSync, writeFileSync } from "node:fs";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  hashRegularFile,
  readRegularFile,
  readRegularTextSync,
  withRegularFileSync,
} from "./regular-file.mjs";

test("lit un fichier régulier avec une borne stricte", async (context) => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-regular-file-"));
  context.after(() => rm(directory, { recursive: true, force: true }));
  const path = join(directory, "source.txt");
  await writeFile(path, "contenu");

  assert.equal(readRegularTextSync(path, 7), "contenu");
  assert.equal((await readRegularFile(path, 7)).toString("utf8"), "contenu");
  await assert.rejects(readRegularFile(path, 6), /validation failed/u);
  assert.deepEqual(await hashRegularFile(path, 7), {
    sha256: "3016ef88e3166466281c563b984abed5412a2de823d37ed99c2af39be422fab1",
    size: 7,
  });
});

test("refuse les fichiers vides et les liens symboliques", async (context) => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-regular-file-"));
  context.after(() => rm(directory, { recursive: true, force: true }));
  const target = join(directory, "target.txt");
  const link = join(directory, "link.txt");
  await writeFile(target, "ok");

  await assert.rejects(readRegularFile(join(directory, "missing.txt"), 16));
  await writeFile(join(directory, "empty.txt"), "");
  await assert.rejects(readRegularFile(join(directory, "empty.txt"), 16));
  try {
    symlinkSync(target, link);
    assert.throws(() => readRegularTextSync(link, 16));
  } catch (error) {
    if (!["EPERM", "EACCES"].includes(error?.code)) throw error;
  }
});

test("refuse le remplacement du chemin pendant une lecture", async (context) => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-regular-file-"));
  context.after(() => rm(directory, { recursive: true, force: true }));
  const path = join(directory, "source.txt");
  const original = join(directory, "original.txt");
  writeFileSync(path, "initial");

  assert.throws(
    () => withRegularFileSync(path, 16, () => {
      renameSync(path, original);
      writeFileSync(path, "remplace");
    }),
    /validation failed/u,
  );
});
