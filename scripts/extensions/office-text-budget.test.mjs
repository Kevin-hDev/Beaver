import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { LIMITS } from "../../src-tauri/resources/extension-host/contract.mjs";
import { OFFICE_LIMITS } from "../../src-tauri/resources/extension-host/builtin-plugins/common/constants.mjs";
import {
  callOffice,
  createOfficeHost,
  syncOfficePlugins,
} from "./office-test-helpers.mjs";

const CJK_BYTES_PER_CHARACTER = 3;

test("the text budget always fits the transport limit", () => {
  assert.ok(OFFICE_LIMITS.maxTextBytes < LIMITS.maxMessageBytes);
  assert.ok(
    OFFICE_LIMITS.maxTextBytes + 2_048 < LIMITS.maxMessageBytes,
    "the byte budget must leave room for the JSON-RPC envelope",
  );
  assert.ok(OFFICE_LIMITS.maxTextChars <= OFFICE_LIMITS.maxTextBytes);
});

test("a Latin document keeps the full character budget", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-budget-latin-"));
  const host = createOfficeHost();
  try {
    await syncOfficePlugins(host);
    const blocks = Math.floor(OFFICE_LIMITS.maxTextChars / 32_000);
    const result = await callOffice(host, workspace, "beaver.office.documents.create", {
      path: "latin.docx",
      blocks: Array.from({ length: blocks }, () => ({
        type: "paragraph",
        text: "a".repeat(32_000),
      })),
    });
    assert.notEqual(result.isError, true);
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});

test("a CJK document over the byte budget is refused before the transport", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-budget-cjk-"));
  const host = createOfficeHost();
  try {
    await syncOfficePlugins(host);
    const characters = Math.floor(
      OFFICE_LIMITS.maxTextBytes / CJK_BYTES_PER_CHARACTER,
    ) + 1_000;
    const blocks = Math.ceil(characters / 32_000);
    const result = await callOffice(host, workspace, "beaver.office.documents.create", {
      path: "cjk.docx",
      blocks: Array.from({ length: blocks }, () => ({
        type: "paragraph",
        text: "界".repeat(32_000),
      })),
    });
    assert.equal(result.isError, true);
    assert.equal(result.content, "input_too_large");
    assert.equal((await host.request("host.hello", {})).apiVersion, "1");
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});

test("every Office tool applies the same text budget", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-budget-tools-"));
  const host = createOfficeHost();
  const oversized = "界".repeat(32_000);
  const blocks = Math.ceil(
    (OFFICE_LIMITS.maxTextBytes / CJK_BYTES_PER_CHARACTER + 1_000) / 32_000,
  );
  try {
    await syncOfficePlugins(host);
    const calls = [
      ["beaver.office.pdf.create", {
        path: "budget.pdf",
        paragraphs: Array.from({ length: blocks }, () => oversized),
      }],
      ["beaver.office.documents.create", {
        path: "budget.docx",
        blocks: Array.from({ length: blocks }, () => ({
          type: "paragraph",
          text: oversized,
        })),
      }],
      ["beaver.office.presentations.create", {
        path: "budget.pptx",
        // Une puce est bornée à 2 000 caractères : il faut plusieurs diapositives
        // pleines pour franchir le budget en octets.
        slides: Array.from({ length: 8 }, () => ({
          title: "Titre",
          bullets: Array.from({ length: 20 }, () => "界".repeat(2_000)),
        })),
      }],
    ];
    for (const [name, args] of calls) {
      const result = await callOffice(host, workspace, name, args);
      assert.equal(result.isError, true, name);
      assert.equal(result.content, "input_too_large", name);
    }
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});
