import { describe, expect, it } from "vitest";
import { XSS_PAYLOADS } from "../test-utils/xss-corpus";
import { highlightLines } from "../highlight";

const EXTENSIONS = ["page.html", "app.js", "example.ts", "style.md"];

describe("highlightLines - batterie XSS", () => {
  for (const ext of EXTENSIONS) {
    for (const payload of XSS_PAYLOADS) {
      it(`neutralise « ${payload.name} » dans ${ext}`, () => {
        const lines = highlightLines(payload.input, ext);

        for (const html of lines) {
          /* Seuls les <span> du colorateur sont des balises légitimes. */
          const withoutSpans = html.replace(/<\/?span[^>]*>/g, "");
          expect(
            withoutSpans,
            `balise survivante dans ${ext} pour « ${payload.name} » : ${withoutSpans.slice(0, 80)}`,
          ).not.toContain("<");
        }
      });
    }
  }
});
