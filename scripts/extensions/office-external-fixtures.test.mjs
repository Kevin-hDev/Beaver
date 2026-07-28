import assert from "node:assert/strict";
import {
  access,
  copyFile,
  mkdtemp,
  readFile,
  rm,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { strFromU8, unzipSync } from "fflate";
import { LIMITS } from "../../src-tauri/resources/extension-host/contract.mjs";
import {
  callOffice,
  createOfficeHost,
  fixtureDirectory,
  syncOfficePlugins,
} from "./office-test-helpers.mjs";

test("bounds an external XLSX preview without losing the extension host", async () => {
  await withFixture("external-large-preview.xlsx", async ({ host, workspace }) => {
    const result = await callOffice(
      host,
      workspace,
      "beaver.office.spreadsheets.inspect",
      { path: "external-large-preview.xlsx", maxRows: 200, maxColumns: 100 },
    );
    const payload = JSON.parse(result.content);

    assert.notEqual(result.isError, true);
    assert.equal(payload.truncated, true);
    assert.ok(Buffer.byteLength(JSON.stringify(result), "utf8") < LIMITS.maxMessageBytes);
    assert.equal((await host.request("host.hello", {})).apiVersion, "1");
  });
});

test("bounds Japanese text extracted from an external PDF", async () => {
  await withFixture("external-large-text.pdf", async ({ host, workspace }) => {
    const result = await callOffice(
      host,
      workspace,
      "beaver.office.pdf.inspect",
      { path: "external-large-text.pdf", maxPages: 20 },
    );
    const payload = JSON.parse(result.content);

    assert.notEqual(result.isError, true);
    assert.equal(payload.truncated, true);
    assert.ok(Buffer.byteLength(JSON.stringify(result), "utf8") < LIMITS.maxMessageBytes);
    assert.equal((await host.request("host.hello", {})).apiVersion, "1");
  });
});

test("patches a placeholder split across real PowerPoint runs", async () => {
  await withFixture("external-fragmented-runs.pptx", async ({ host, workspace }) => {
    const result = await callOffice(
      host,
      workspace,
      "beaver.office.presentations.patch",
      {
        sourcePath: "external-fragmented-runs.pptx",
        outputPath: "patched.pptx",
        replacements: { customer: "Beaver & Co" },
      },
    );
    assert.notEqual(result.isError, true);
    const files = unzipSync(await readFile(join(workspace, "patched.pptx")));
    const xml = strFromU8(files["ppt/slides/slide1.xml"]);
    assert.equal(powerPointText(xml).includes("Prepared for Beaver & Co"), true);
    assert.equal(xml.includes("{{customer}}"), false);
  });
});

test("rejects XML control characters before writing a presentation", async () => {
  await withFixture("external-fragmented-runs.pptx", async ({ host, workspace }) => {
    const result = await callOffice(
      host,
      workspace,
      "beaver.office.presentations.patch",
      {
        sourcePath: "external-fragmented-runs.pptx",
        outputPath: "invalid.pptx",
        replacements: { customer: "bad\u0001value" },
      },
    );
    assert.equal(result.isError, true);
    assert.equal(result.content, "invalid_input");
    await assert.rejects(access(join(workspace, "invalid.pptx")));
  });
});

async function withFixture(name, assertion) {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-external-office-"));
  const host = createOfficeHost();
  try {
    await copyFile(join(fixtureDirectory, name), join(workspace, name));
    await syncOfficePlugins(host);
    await assertion({ host, workspace });
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
}

function powerPointText(xml) {
  return [...xml.matchAll(/<a:t(?:\s[^>]*)?>([\s\S]*?)<\/a:t>/gu)]
    .map((match) => match[1])
    .join("")
    .replaceAll("&amp;", "&")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">");
}
