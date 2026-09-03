import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, mkdir, readFile, realpath, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { afterEach, test } from "node:test";

import { buildExtensionUi } from "./ui-build.mjs";

const execute = promisify(execFile);
const script = resolve(dirname(fileURLToPath(import.meta.url)), "ui-build.mjs");
const roots = [];

afterEach(async () => {
  const { rm } = await import("node:fs/promises");
  await Promise.all(roots.splice(0).map((root) => rm(root, { recursive: true, force: true })));
});

async function fixture() {
  const root = await realpath(await mkdtemp(join(tmpdir(), "beaver-ui-build-")));
  roots.push(root);
  const inputRoot = join(root, "input");
  const outputRoot = join(root, "output");
  await Promise.all([mkdir(inputRoot), mkdir(outputRoot)]);
  return { root, inputRoot, outputRoot };
}

test("bundles JS, TS, TSX, CSS, raster images, WOFF2 and a local npm dependency", async () => {
  const { inputRoot, outputRoot } = await fixture();
  await mkdir(join(inputRoot, "node_modules", "tiny-package"), { recursive: true });
  await writeFile(join(inputRoot, "node_modules/tiny-package/package.json"), '{"type":"module","exports":"./index.js"}');
  await writeFile(join(inputRoot, "node_modules/tiny-package/index.js"), "export const answer = 42;");
  await writeFile(join(inputRoot, "style.css"), '@font-face{font-family:x;src:url("./font.woff2")} .icon{background:url("./pixel.png")}');
  await writeFile(join(inputRoot, "font.woff2"), Buffer.from("wOF2fixture"));
  await writeFile(join(inputRoot, "pixel.png"), Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]));
  await writeFile(join(inputRoot, "component.tsx"), "export const value: number = 7;");
  await writeFile(join(inputRoot, "entry.ts"), 'import "./style.css"; import {value} from "./component"; import {answer} from "tiny-package"; export const total=value+answer;');

  const manifest = await buildExtensionUi({ inputRoot, outputRoot, entry: "entry.ts" });

  assert.equal(manifest.version, 1);
  assert.ok(manifest.outputs.some((output) => output.type === "javascript"));
  assert.ok(manifest.outputs.some((output) => output.type === "css"));
  assert.ok(manifest.outputs.some((output) => output.type === "png"));
  assert.ok(manifest.outputs.some((output) => output.type === "woff2"));
  assert.ok(manifest.inputs.includes("node_modules/tiny-package/index.js"));
  assert.match(manifest.manifestSha256, /^[a-f0-9]{64}$/u);
  for (const output of manifest.outputs) {
    assert.match(output.sha256, /^[a-f0-9]{64}$/u);
    assert.ok((await readFile(join(outputRoot, output.name))).length > 0);
  }
});

test("rejects Node imports, traversal, symlinks and unknown output types", async () => {
  const { root, inputRoot, outputRoot } = await fixture();
  await writeFile(join(inputRoot, "node.ts"), 'import "node:fs";');
  await assert.rejects(() => buildExtensionUi({ inputRoot, outputRoot, entry: "node.ts" }), /Extension UI build failed/u);

  await writeFile(join(root, "outside.ts"), "export default 1;");
  await assert.rejects(() => buildExtensionUi({ inputRoot, outputRoot, entry: "../outside.ts" }), /Extension UI build failed/u);

  await symlink(join(root, "outside.ts"), join(inputRoot, "linked.ts"));
  await assert.rejects(() => buildExtensionUi({ inputRoot, outputRoot, entry: "linked.ts" }), /Extension UI build failed/u);

  await writeFile(join(inputRoot, "icon.svg"), "<svg/>");
  await writeFile(join(inputRoot, "svg.ts"), 'import icon from "./icon.svg"; export default icon;');
  await assert.rejects(() => buildExtensionUi({ inputRoot, outputRoot, entry: "svg.ts" }), /Extension UI build failed/u);
});

test("rejects more than 64 outputs and artifacts larger than four MiB", async () => {
  const { inputRoot, outputRoot } = await fixture();
  const imports = [];
  for (let index = 0; index < 65; index += 1) {
    const name = `asset-${index}.png`;
    await writeFile(join(inputRoot, name), Buffer.from([137, 80, 78, 71, index]));
    imports.push(`import asset${index} from "./${name}";`);
  }
  imports.push(`export const assets = [${Array.from({ length: 65 }, (_, index) => `asset${index}`).join(",")}];`);
  await writeFile(join(inputRoot, "many.ts"), imports.join("\n"));
  await assert.rejects(() => buildExtensionUi({ inputRoot, outputRoot, entry: "many.ts" }), /Extension UI build failed/u);

  await writeFile(join(inputRoot, "large.bin"), Buffer.alloc(4_194_305, 1));
  await writeFile(join(inputRoot, "large.ts"), 'import data from "./large.bin"; export default data;');
  await assert.rejects(() => buildExtensionUi({ inputRoot, outputRoot, entry: "large.ts" }), /Extension UI build failed/u);
});

test("CLI writes one bounded JSON value and keeps failures generic", async () => {
  const { root, inputRoot, outputRoot } = await fixture();
  await writeFile(join(inputRoot, "entry.js"), "export const ready = true;");
  const success = await execute(process.execPath, [script, "--input-root", inputRoot, "--output-root", outputRoot, "--entry", "entry.js"]);
  assert.doesNotThrow(() => JSON.parse(success.stdout));
  assert.ok(Buffer.byteLength(success.stdout) < 1_048_576);

  await assert.rejects(
    execute(process.execPath, [script, "--input-root", inputRoot, "--output-root", outputRoot, "--entry", "../missing.js"]),
    (error) => {
      assert.equal(error.stdout, "");
      assert.equal(error.stderr, "Extension UI build failed\n");
      assert.ok(!error.stderr.includes(root));
      return true;
    },
  );
});
