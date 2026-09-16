import { describe, expect, it } from "vitest";
import { insertAtSelection } from "../composer-draft-insertion";

describe("insertAtSelection", () => {
  it("insère au début d'une sélection sans remplacer le texte", () => {
    expect(insertAtSelection("déjà ici", "DICTÉ", { anchor: 8, head: 5 }))
      .toEqual({ text: "déjà DICTÉici", cursor: 10 });
  });

  it("ajoute à la fin quand la position est périmée", () => {
    expect(insertAtSelection("😀", " suite", { anchor: 99, head: 99 }))
      .toEqual({ text: "😀 suite", cursor: 8 });
  });

  it("refuse une position qui coupe une paire de substitution", () => {
    expect(insertAtSelection("a😀b", "X", { anchor: 2, head: 2 }))
      .toEqual({ text: "a😀bX", cursor: 5 });
  });

  it("n'invente aucun espace", () => {
    expect(insertAtSelection("ab", "X", { anchor: 1, head: 1 }))
      .toEqual({ text: "aXb", cursor: 2 });
  });
});
