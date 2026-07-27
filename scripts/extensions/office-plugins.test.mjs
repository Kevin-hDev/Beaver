import assert from "node:assert/strict";
import { access, mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { strFromU8, unzipSync, zipSync } from "fflate";
import { createHost } from "./host-test-client.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const hostDirectory = join(root, "src-tauri/target/extension-host");
const hostScript = join(hostDirectory, "host.mjs");
const expectedDependencies = {
  "@xlsx/xlsx-populate": "0.2.0",
  docx: "9.7.1",
  fflate: "0.8.3",
  jiti: "2.7.0",
  "pdf-lib": "1.17.1",
  "pdfjs-dist": "6.1.200",
  pptxgenjs: "4.0.1",
};

test("the Office runtime dependencies are explicit and exactly pinned", async () => {
  const manifest = JSON.parse(
    await readFile(join(hostDirectory, "package.json"), "utf8"),
  );
  assert.deepEqual(manifest.dependencies, expectedDependencies);
});

test("the Office archive guard rejects suspicious expansion ratios", async () => {
  const guardUrl = pathToFileURL(
    join(hostDirectory, "builtin-plugins/common/zip-guard.mjs"),
  );
  const { assertSafeOfficeArchive } = await import(guardUrl.href);
  const archive = Buffer.from(zipSync({
    "word/document.xml": new Uint8Array(2 * 1024 * 1024),
  }));

  try {
    assert.throws(
      () => assertSafeOfficeArchive(archive),
      (error) => error?.code === "unsafe_archive",
    );
  } finally {
    archive.fill(0);
  }
});

test("the bundled Office suite loads and creates real local artifacts", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-office-test-"));
  const host = createHost(hostScript);
  try {
    const sync = await syncOfficePlugins(host);
    assert.equal(sync.extensions.length, 4);
    assert.equal(sync.extensions.every((extension) => !extension.error), true);
    assert.equal(
      sync.extensions.reduce(
        (count, extension) => count + extension.contributions.tools.length,
        0,
      ),
      10,
    );

    const createdDocument = await call(
      host,
      workspace,
      "beaver.office.documents.create",
      {
        path: join(workspace, "report.docx"),
        title: "Quarterly report",
        blocks: [
          { type: "heading", level: 1, text: "Overview" },
          { type: "paragraph", text: "The document plugin is active." },
          { type: "paragraph", text: "Prepared for {{name}}" },
        ],
      },
    );
    assert.notEqual(createdDocument.isError, true);
    assert.equal(
      (await readFile(join(workspace, "report.docx")))
        .subarray(0, 2)
        .toString(),
      "PK",
    );
    await call(host, workspace, "beaver.office.documents.patch", {
      sourcePath: "report.docx",
      outputPath: "filled-report.docx",
      replacements: { name: "Beaver" },
    });
    const documentFiles = unzipSync(
      await readFile(join(workspace, "filled-report.docx")),
    );
    const documentXml = strFromU8(documentFiles["word/document.xml"]);
    assert.equal(
      documentXml.includes("Prepared for ")
        && documentXml.includes("Beaver")
        && !documentXml.includes("{{name}}"),
      true,
    );

    await call(host, workspace, "beaver.office.pdf.create", {
      path: "report.pdf",
      title: "Quarterly report",
      paragraphs: ["The PDF plugin is active."],
    });
    const pdf = await readFile(join(workspace, "report.pdf"));
    assert.equal(pdf.subarray(0, 5).toString(), "%PDF-");
    const inspectedPdf = await call(host, workspace, "beaver.office.pdf.inspect", {
      path: "report.pdf",
    });
    assert.equal(JSON.parse(inspectedPdf.content).pageCount, 1);
    await call(host, workspace, "beaver.office.pdf.create", {
      path: "appendix.pdf",
      paragraphs: ["Appendix"],
    });
    await call(host, workspace, "beaver.office.pdf.merge", {
      sourcePaths: ["report.pdf", "appendix.pdf"],
      outputPath: "merged.pdf",
    });
    const mergedPdf = await call(host, workspace, "beaver.office.pdf.inspect", {
      path: "merged.pdf",
    });
    assert.equal(JSON.parse(mergedPdf.content).pageCount, 2);

    await call(host, workspace, "beaver.office.spreadsheets.create", {
      path: "data.xlsx",
      sheets: [{ name: "Data", rows: [["Name", "Value"], ["Beaver", 42]] }],
    });
    await call(host, workspace, "beaver.office.spreadsheets.update", {
      sourcePath: "data.xlsx",
      outputPath: "updated.xlsx",
      changes: [{ sheet: "Data", cell: "B2", value: 84 }],
    });
    const inspectedSheet = await call(
      host,
      workspace,
      "beaver.office.spreadsheets.inspect",
      { path: "updated.xlsx" },
    );
    assert.equal(JSON.parse(inspectedSheet.content).sheets[0].preview[1][1], 84);

    await call(host, workspace, "beaver.office.presentations.create", {
      path: "deck.pptx",
      slides: [{ title: "Hello {{name}}", bullets: ["First point"] }],
    });
    await call(host, workspace, "beaver.office.presentations.patch", {
      sourcePath: "deck.pptx",
      outputPath: "patched.pptx",
      replacements: { name: "Beaver" },
    });
    const slides = unzipSync(await readFile(join(workspace, "patched.pptx")));
    const slideXml = strFromU8(slides["ppt/slides/slide1.xml"]);
    assert.equal(slideXml.includes("Hello Beaver"), true);
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});

test("official plugins reject paths outside the active workspace", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-office-path-test-"));
  const outside = join(workspace, "..", "beaver-office-escape.docx");
  const host = createHost(hostScript);
  try {
    await syncOfficePlugins(host);
    const result = await call(host, workspace, "beaver.office.documents.create", {
      path: "../beaver-office-escape.docx",
      blocks: [{ type: "paragraph", text: "blocked" }],
    });
    assert.equal(result.isError, true);
    assert.equal(result.content, "invalid_path");
    await assert.rejects(access(outside));
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});

async function syncOfficePlugins(host) {
  const catalog = JSON.parse(
    await readFile(join(hostDirectory, "builtin-plugins/catalog.json"), "utf8"),
  );
  return host.request("host.sync", {
    extensions: catalog.plugins.map(({ manifest }) => ({
      id: manifest.id,
      mainPath: join(hostDirectory, manifest.main),
      manifest,
    })),
  });
}

async function call(host, workspace, name, arguments_) {
  return host.request("tool.call", {
    name,
    arguments: arguments_,
    context: { workingDirectory: workspace },
  });
}
