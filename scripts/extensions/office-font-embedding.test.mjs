import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { PDFDict, PDFDocument, PDFName } from "@cantoo/pdf-lib";
import {
  callOffice,
  createOfficeHost,
  syncOfficePlugins,
} from "./office-test-helpers.mjs";

test("embeds only fonts selected after fallback probing", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-font-probe-pdf-"));
  const host = createOfficeHost();
  try {
    await syncOfficePlugins(host);
    const created = await callOffice(host, workspace, "beaver.office.pdf.create", {
      path: "math.pdf",
      paragraphs: ["a ≤ b ≠ c"],
    });
    assert.notEqual(created.isError, true);
    const bytes = await readFile(join(workspace, "math.pdf"));
    const document = await PDFDocument.load(bytes);
    try {
      const type = PDFName.of("Type");
      const fonts = document.context.enumerateIndirectObjects()
        .filter(([, object]) =>
          object instanceof PDFDict
          && object.get(type)?.toString() === "/Font");
      assert.ok(fonts.length <= 4);
    } finally {
      bytes.fill(0);
    }
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});
