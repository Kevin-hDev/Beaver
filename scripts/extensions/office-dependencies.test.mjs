import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { access, readFile, readdir } from "node:fs/promises";
import { createRequire } from "node:module";
import { join, relative, sep } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hostDirectory, root } from "./office-test-helpers.mjs";

const expectedDependencies = {
  "@napi-rs/canvas": "1.0.8",
  "@cantoo/pdf-lib": "2.9.1",
  "@xlsx/xlsx-populate": "0.2.0",
  docx: "9.7.1",
  fflate: "0.8.3",
  fontkit: "2.0.4",
  "image-size": "file:vendor/image-size-disabled",
  jiti: "2.7.0",
  "pdfjs-dist": "6.2.108",
  pptxgenjs: "4.0.1",
};

test("the production Office tree is exact and pinned", async () => {
  const [hostManifest, rootManifest] = await Promise.all([
    jsonFile(join(hostDirectory, "package.json")),
    jsonFile(join(root, "package.json")),
  ]);

  assert.deepEqual(hostManifest.dependencies, expectedDependencies);
  for (const [name, version] of Object.entries(expectedDependencies)) {
    const rootVersion = name === "image-size"
      ? "file:src-tauri/resources/extension-host/vendor/image-size-disabled"
      : version;
    assert.equal(rootManifest.devDependencies[name], rootVersion);
    assert.equal(rootManifest.dependencies[name], undefined);
  }
  await access(join(hostDirectory, "node_modules/@napi-rs/canvas"));
  const httpsManifest = await jsonFile(
    join(hostDirectory, "node_modules/pptxgenjs/node_modules/https/package.json"),
  );
  assert.equal(httpsManifest.name, "https-browserify");
  assert.equal(httpsManifest.version, "1.0.0");
});

test("the disabled image inspector fails closed", async () => {
  const hostRequire = createRequire(join(hostDirectory, "package.json"));
  const imageSize = hostRequire("image-size");

  assert.throws(
    () => imageSize(new Uint8Array([0, 0, 0, 0])),
    /image_inspection_disabled/u,
  );
  assert.deepEqual([...imageSize.types], []);
});

test("the prepared PDF runtime provides native DOM geometry", async () => {
  const canvasPath = join(
    hostDirectory,
    "node_modules/@napi-rs/canvas/index.js",
  );
  const canvas = await import(pathToFileURL(canvasPath).href);

  assert.equal(typeof canvas.DOMMatrix, "function");
  assert.equal(typeof canvas.Path2D, "function");
  assert.equal(typeof canvas.ImageData, "function");
});

test("format libraries are isolated behind Beaver adapters", async () => {
  const pluginRoot = join(hostDirectory, "builtin-plugins");
  const files = await sourceFiles(pluginRoot, 128);
  const adapters = `${join("common", "formats")}${sep}`;
  const packageImports = [
    "@cantoo/pdf-lib",
    "@xlsx/xlsx-populate",
    "docx",
    "fflate",
    "fontkit",
    "pdfjs-dist",
    "pptxgenjs",
  ];
  for (const file of files) {
    if (relative(pluginRoot, file).startsWith(adapters)) continue;
    const source = await readFile(file, "utf8");
    for (const packageName of packageImports) {
      assert.equal(source.includes(`from "${packageName}`), false, file);
    }
  }
});

test("the PDF fork does not restore the discontinued crypto-js dependency", async () => {
  const sourceRoot = join(
    hostDirectory,
    "node_modules/@cantoo/pdf-lib/src",
  );
  const files = await sourceFiles(sourceRoot, 2_048);
  const imports = [];
  for (const file of files) {
    if ((await readFile(file, "utf8")).includes("crypto-js")) imports.push(file);
  }
  assert.deepEqual(imports, []);
});

test("bundled Unicode fonts match their pinned upstream bytes", async () => {
  const fontRoot = join(hostDirectory, "builtin-plugins/common/fonts");
  const catalogPath = join(fontRoot, "catalog.mjs");
  const { PDF_FONT_SPECS } = await import(pathToFileURL(catalogPath).href);
  const fontFiles = (await readdir(fontRoot))
    .filter((file) => /\.(?:otf|ttf)$/u.test(file))
    .sort();
  assert.deepEqual(
    Object.values(PDF_FONT_SPECS).map(({ file }) => file).sort(),
    fontFiles,
  );
  for (const spec of Object.values(PDF_FONT_SPECS)) {
    const bytes = await readFile(join(fontRoot, spec.file));
    assert.equal(bytes.length, spec.bytes, spec.file);
    assert.equal(sha256(bytes), spec.sha256, spec.file);
  }
});

async function jsonFile(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function sourceFiles(directory, maximum) {
  const files = [];
  const pending = [directory];
  while (pending.length > 0) {
    const current = pending.pop();
    for (const entry of await readdir(current, { withFileTypes: true })) {
      const path = join(current, entry.name);
      if (entry.isDirectory()) pending.push(path);
      else if (entry.isFile() && /\.(?:mjs|js|ts)$/u.test(entry.name)) files.push(path);
      if (files.length + pending.length > maximum) throw new Error("file_limit");
    }
  }
  return files.sort();
}
