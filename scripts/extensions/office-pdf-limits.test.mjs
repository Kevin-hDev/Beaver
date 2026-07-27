import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  callOffice,
  createOfficeHost,
  syncOfficePlugins,
} from "./office-test-helpers.mjs";

test("stops PDF pagination at the configured page limit", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-pdf-pages-"));
  const host = createOfficeHost();
  try {
    await syncOfficePlugins(host);
    const result = await callOffice(host, workspace, "beaver.office.pdf.create", {
      path: "too-many-pages.pdf",
      paragraphs: [Array.from({ length: 10_000 }, () => "x").join("\n")],
    });
    assert.equal(result.isError, true);
    assert.equal(result.content, "output_too_large");
    assert.equal((await host.request("host.hello", {})).apiVersion, "1");
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});
