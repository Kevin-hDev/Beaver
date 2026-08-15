import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync("src/components/settings/system-prompt-settings.css", "utf8");

/* La carte doit occuper la hauteur restante et garder la même en lecture comme
   en édition. Chaque déclaration ci-dessous porte une moitié de ce contrat :
   sans la première la carte déborde sous la fenêtre, sans la seconde la zone de
   texte impose sa propre hauteur et les deux modes cessent de coïncider. */
describe("hauteur de la carte des instructions système", () => {
  it("empêche la page de défiler à la place de la zone de texte", () => {
    expect(css).toContain("overflow: hidden;");
  });

  it("donne la même règle de hauteur à la lecture et à l'édition", () => {
    const shared = css.slice(css.indexOf(".sps-page .spp-preview,"));
    expect(shared).toContain(".spp-textarea {");
    expect(shared).toContain("max-height: none;");
    expect(shared).toContain("flex: 1;");
  });

  it("interdit le redimensionnement manuel de la zone d'édition", () => {
    expect(css).toContain("resize: none;");
  });
});
