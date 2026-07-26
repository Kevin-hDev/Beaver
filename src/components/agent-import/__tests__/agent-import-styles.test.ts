import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const cssPath = resolve(
  process.cwd(),
  "src/components/agent-import/agent-import.css",
);

describe("styles de la migration des assistants", () => {
  it("conserve la même graisse pour les boutons principaux", () => {
    // Le chemin est une constante interne propre au test.
    // eslint-disable-next-line security/detect-non-literal-fs-filename
    const css = readFileSync(cssPath, "utf8");

    expect(css).toMatch(/\.aim-btn-primary\s*\{[^}]*font-weight:\s*700;/s);
  });
});
