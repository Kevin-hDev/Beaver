import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { PDFDocument } from "@cantoo/pdf-lib";
import { PDF_FONT_SPECS } from "../../src-tauri/resources/extension-host/builtin-plugins/common/fonts/catalog.mjs";
import { fontkit } from "../../src-tauri/resources/extension-host/builtin-plugins/common/formats/pdf.mjs";

test("pdf-lib still consumes Beaver's fontkit encodeStream adapter", async () => {
  const bytes = await readFile(PDF_FONT_SPECS.base.url);
  let encodeStreamCalls = 0;
  const monitoredFontkit = {
    ...fontkit,
    create(...arguments_) {
      const font = fontkit.create(...arguments_);
      const createSubset = font.createSubset.bind(font);
      font.createSubset = () => {
        const subset = createSubset();
        const encodeStream = subset.encodeStream.bind(subset);
        subset.encodeStream = () => {
          encodeStreamCalls += 1;
          return encodeStream();
        };
        return subset;
      };
      return font;
    },
  };
  try {
    const document = await PDFDocument.create();
    document.registerFontkit(monitoredFontkit);
    const embedded = await document.embedFont(bytes, { subset: true });
    document.addPage().drawText("Beaver", { font: embedded });
    const output = await document.save();
    assert.ok(output.length > 0);
    assert.ok(encodeStreamCalls > 0);
    output.fill(0);
  } finally {
    bytes.fill(0);
  }
});
