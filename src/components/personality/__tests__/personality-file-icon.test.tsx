/* @vitest-environment jsdom */
import { readFileSync } from "node:fs";
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { PersonalityFileIcon } from "../personality-file-icon";

const TOKENS = readFileSync("src/styles/tokens-icon-sizes.css", "utf8");

afterEach(() => {
  cleanup();
});

describe("dessin devant un fichier de /beaver", () => {
  /* Ce dessin est le seul de l'application posé au-dessus de l'icône standard :
     le ramener au format commun le rendrait maigre en face de deux lignes de
     texte, et le monter davantage rognerait le plus long des cinq noms. */
  it("se pose un cinquième au-dessus de l'icône standard", () => {
    const { container } = render(<PersonalityFileIcon />);
    const svg = container.querySelector("svg");

    expect(svg?.style.width).toBe("var(--personality-file-icon-size)");
    expect(TOKENS).toContain("--personality-file-icon-size: calc(var(--icon-md) * 1.2);");
  });
});
