import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { access, readFile, readdir } from "node:fs/promises";
import { join, relative } from "node:path";
import { test } from "node:test";
import { hostDirectory, root } from "./office-test-helpers.mjs";

const expectedDependencies = {
  "@cantoo/pdf-lib": "2.7.4",
  "@pdf-lib/fontkit": "1.1.1",
  "@xlsx/xlsx-populate": "0.2.0",
  docx: "9.7.1",
  fflate: "0.8.3",
  jiti: "2.7.0",
  "pdfjs-dist": "6.1.200",
  pptxgenjs: "4.0.1",
};

test("the production Office tree is exact, pinned, and omits optional canvas", async () => {
  const [hostManifest, rootManifest] = await Promise.all([
    jsonFile(join(hostDirectory, "package.json")),
    jsonFile(join(root, "package.json")),
  ]);

  assert.deepEqual(hostManifest.dependencies, expectedDependencies);
  for (const [name, version] of Object.entries(expectedDependencies)) {
    assert.equal(rootManifest.devDependencies[name], version);
    assert.equal(rootManifest.dependencies[name], undefined);
  }
  await assert.rejects(
    access(join(hostDirectory, "node_modules/@napi-rs/canvas")),
  );
  const httpsManifest = await jsonFile(
    join(hostDirectory, "node_modules/pptxgenjs/node_modules/https/package.json"),
  );
  assert.equal(httpsManifest.name, "https-browserify");
  assert.equal(httpsManifest.version, "1.0.0");
});

test("format libraries are isolated behind Beaver adapters", async () => {
  const pluginRoot = join(hostDirectory, "builtin-plugins");
  const files = await sourceFiles(pluginRoot, 128);
  const adapters = `${join("common", "formats")}/`;
  const packageImports = [
    "@cantoo/pdf-lib",
    "@pdf-lib/fontkit",
    "@xlsx/xlsx-populate",
    "docx",
    "fflate",
    "pdfjs-dist",
    "pptxgenjs",
  ];
  for (const file of files) {
    if (relative(pluginRoot, file).includes(adapters)) continue;
    const source = await readFile(file, "utf8");
    for (const packageName of packageImports) {
      assert.equal(source.includes(`from "${packageName}`), false, file);
    }
  }
});

test("the PDF fork confines crypto-js imports to its encryption module", async () => {
  const sourceRoot = join(
    hostDirectory,
    "node_modules/@cantoo/pdf-lib/src",
  );
  const files = await sourceFiles(sourceRoot, 2_048);
  const imports = [];
  for (const file of files) {
    if ((await readFile(file, "utf8")).includes("crypto-js")) imports.push(file);
  }
  assert.deepEqual(
    imports.map((file) => relative(sourceRoot, file)),
    [join("core", "security", "PDFSecurity.ts")],
  );
});

test("bundled Unicode fonts match their pinned upstream bytes", async () => {
  const fontRoot = join(hostDirectory, "builtin-plugins/common/fonts");
  assert.equal(
    await sha256(join(fontRoot, "NotoSansCJKjp-Regular.otf")),
    "68a3fc98800b2a27b371f2fb79991daf3633bd89309d4ffaa6946fd587f375b5",
  );
  assert.equal(
    await sha256(join(fontRoot, "NotoSansArabic-Regular.ttf")),
    "ceea25b464a656dc3b26849bab9356740401af62aedf1bfa8b7f0d9b75925b1b",
  );
});

async function jsonFile(path) {
  return JSON.parse(await readFile(path, "utf8"));
}

async function sha256(path) {
  return createHash("sha256").update(await readFile(path)).digest("hex");
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
