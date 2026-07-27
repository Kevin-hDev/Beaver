import { PassThrough } from "node:stream";
import * as currentFontkit from "fontkit";

const RTL_TEXT = /[\p{Script=Arabic}\p{Script=Hebrew}]/u;

export {
  beginMarkedContent,
  endMarkedContent,
  PDFDocument,
  rgb,
  TextRenderingMode,
} from "@cantoo/pdf-lib";

export const fontkit = Object.freeze({
  ...currentFontkit,
  create(fontData, postscriptName) {
    return compatibleFont(fontData, postscriptName, false);
  },
});

export const logicalFontkit = Object.freeze({
  ...currentFontkit,
  create(fontData, postscriptName) {
    return compatibleFont(fontData, postscriptName, true);
  },
});

function compatibleSubset(subset) {
  subset.encodeStream = () => {
    const stream = new PassThrough();
    queueMicrotask(() => {
      try {
        stream.end(Buffer.from(subset.encode()));
      } catch (error) {
        stream.destroy(error);
      }
    });
    return stream;
  };
  return subset;
}

function compatibleFont(fontData, postscriptName, logical) {
  const font = currentFontkit.create(fontData, postscriptName);
  const createSubset = font.createSubset.bind(font);
  font.createSubset = () => compatibleSubset(createSubset());
  if (logical) {
    font.layout = (text) => {
      const glyphs = font.glyphsForString(text);
      return { glyphs: RTL_TEXT.test(text) ? glyphs.reverse() : glyphs };
    };
  }
  return font;
}
