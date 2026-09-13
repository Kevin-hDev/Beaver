import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync("src/components/updates/window/update-progress-window.css", "utf8");
const rowCss = readFileSync("src/components/updates/window/update-operation-row.css", "utf8");
const windowSource = readFileSync("src-tauri/src/services/update_progress/window.rs", "utf8");

describe("update progress window layout", () => {
  it("réserve la place du contenu et de l'ombre sans tronquer les coins", () => {
    expect(windowSource).toContain("pub const WIDTH: f64 = 464.0;");
    expect(windowSource).toContain(".shadow(false)");
    expect(css).toMatch(/\.upw-frame\s*\{[^}]*padding:\s*var\(--chrome-1\) var\(--chrome-3\) var\(--chrome-5\);/s);
    expect(css).toMatch(/\.upw-window\s*\{[^}]*width:\s*100%;/s);
    expect(rowCss).toMatch(/\.upw-measure\s*\{[^}]*min-height:/s);
  });
});
