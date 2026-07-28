import { PassThrough } from "node:stream";
import * as currentFontkit from "fontkit";

export {
  endMarkedContent,
  PDFDocument,
  rgb,
} from "@cantoo/pdf-lib";
export { createTaggedText } from "./pdf-tags.mjs";

export const fontkit = Object.freeze({
  ...currentFontkit,
  create(fontData, postscriptName) {
    return compatibleFont(fontData, postscriptName);
  },
});

function compatibleSubset(subset) {
  // @cantoo/pdf-lib attend l'ancien contrat fontkit encodeStream(), alors que
  // fontkit 2 expose encode(). Cet adaptateur garde le flux asynchrone attendu.
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

function compatibleFont(fontData, postscriptName) {
  const font = currentFontkit.create(fontData, postscriptName);
  const createSubset = font.createSubset.bind(font);
  font.createSubset = () => compatibleSubset(createSubset());
  return font;
}
