import * as currentFontkit from "fontkit";

export {
  endMarkedContent,
  PDFDocument,
  rgb,
} from "@cantoo/pdf-lib";
export { createTaggedText } from "./pdf-tags.mjs";

// pdf-lib 2.9 consumes fontkit 2's native encode() contract directly.
export const fontkit = Object.freeze({ ...currentFontkit });
