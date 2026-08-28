import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { PDFDocument } from "@cantoo/pdf-lib";
import { PDF_FONT_SPECS } from "../../src-tauri/resources/extension-host/builtin-plugins/common/fonts/catalog.mjs";
import { fontkit } from "../../src-tauri/resources/extension-host/builtin-plugins/common/formats/pdf.mjs";

test("pdf-lib consumes fontkit 2's native subset encoder", async () => {
  const bytes = await readFile(PDF_FONT_SPECS.base.url);
  let encodeCalls = 0;
  const monitoredFontkit = {
    ...fontkit,
    create(...arguments_) {
      const font = fontkit.create(...arguments_);
      const createSubset = font.createSubset.bind(font);
      font.createSubset = () => {
        const subset = createSubset();
        const encode = subset.encode.bind(subset);
        subset.encode = () => {
          encodeCalls += 1;
          return encode();
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
    assert.ok(encodeCalls > 0);
    output.fill(0);
  } finally {
    bytes.fill(0);
  }
});
