import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  callOffice,
  createOfficeHost,
  fixtureDirectory,
  syncOfficePlugins,
} from "./office-test-helpers.mjs";

test("creates an offline Unicode PDF across Beaver's supported scripts", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-unicode-pdf-"));
  const host = createOfficeHost();
  try {
    await syncOfficePlugins(host);
    const text = await readFile(
      join(fixtureDirectory, "external-unicode-samples.txt"),
      "utf8",
    );
    const created = await callOffice(host, workspace, "beaver.office.pdf.create", {
      path: "unicode.pdf",
      title: text,
      paragraphs: [text, "界".repeat(500)],
    });
    assert.notEqual(created.isError, true);
    assert.equal((await readFile(join(workspace, "unicode.pdf"))).subarray(0, 5).toString(), "%PDF-");

    const inspected = await callOffice(host, workspace, "beaver.office.pdf.inspect", {
      path: "unicode.pdf",
      maxPages: 20,
    });
    assert.notEqual(inspected.isError, true);
    const extracted = JSON.parse(inspected.content).pages
      .map((page) => page.text)
      .join(" ");
    for (const sample of ["日本語", "中文", "한국어", "Русский", "العربية"]) {
      assert.equal(extracted.includes(sample), true, sample);
    }
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});
