import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync("src/components/settings/font-size-control.css", "utf8");
const rule = css.slice(css.indexOf(".fsc-input {"), css.indexOf("}", css.indexOf(".fsc-input {")));

/* Le champ affiche le réglage de taille de l'interface : son texte grandit avec
   lui jusqu'à 24 px, alors que la hauteur commune des champs est en pixels
   fixes. Sans ces trois déclarations, la ligne finit par dépasser les 26 px
   intérieurs et le nombre se décale. */
describe("hauteur du champ de taille de police", () => {
  it("laisse la hauteur suivre le texte", () => {
    expect(rule).toContain("height: auto;");
  });

  it("ne descend jamais sous la hauteur commune des champs", () => {
    expect(rule).toContain("min-height: var(--btn-height);");
  });

  it("fixe l'interligne, pour un centrage identique d'un moteur à l'autre", () => {
    expect(rule).toContain("line-height: 1.2;");
  });
});
