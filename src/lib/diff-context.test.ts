import { describe, expect, it } from "vitest";
import { extractDiffContext } from "./diff-context";

const FICHIER_10_LIGNES = Array.from({ length: 10 }, (_, i) => `ligne ${i + 1}`).join("\n");
// "ligne 1\nligne 2\n...\nligne 10"

describe("extractDiffContext", () => {
  it("extrait 3 lignes de contexte avant et 3 après l'édition", () => {
    // édition à la ligne 5, 1 ligne remplacée
    const ctx = extractDiffContext(FICHIER_10_LIGNES, 5, 1);
    expect(ctx.before).toEqual(["ligne 2", "ligne 3", "ligne 4"]);
    expect(ctx.after).toEqual(["ligne 6", "ligne 7", "ligne 8"]);
  });

  it("édition au début du fichier (startLine=1) — pas de lignes before", () => {
    const ctx = extractDiffContext(FICHIER_10_LIGNES, 1, 1);
    expect(ctx.before).toEqual([]);
    expect(ctx.after).toEqual(["ligne 2", "ligne 3", "ligne 4"]);
  });

  it("édition à la fin du fichier — pas de lignes after", () => {
    const ctx = extractDiffContext(FICHIER_10_LIGNES, 10, 1);
    expect(ctx.after).toEqual([]);
    expect(ctx.before).toEqual(["ligne 7", "ligne 8", "ligne 9"]);
  });

  it("beforeStartLine et afterStartLine sont corrects", () => {
    // startLine=5, 1 ligne → bStart=1 (index) → beforeStartLine=2
    // editEnd=5 (index) → afterStartLine=6
    const ctx = extractDiffContext(FICHIER_10_LIGNES, 5, 1);
    expect(ctx.beforeStartLine).toBe(2);
    expect(ctx.afterStartLine).toBe(6);
  });

  it("newLineCount=0 (suppression pure) — afterStartLine = startLine", () => {
    // startLine=5, newLineCount=0 → editEnd = 4 (index) = startLine-1
    // after commence à l'index 4, afterStartLine = 5
    const ctx = extractDiffContext(FICHIER_10_LIGNES, 5, 0);
    expect(ctx.afterStartLine).toBe(5);
    expect(ctx.after[0]).toBe("ligne 5");
  });

  it("fichier d'une seule ligne — before et after vides", () => {
    const ctx = extractDiffContext("seule ligne", 1, 1);
    expect(ctx.before).toEqual([]);
    expect(ctx.after).toEqual([]);
    expect(ctx.beforeStartLine).toBe(1);
    expect(ctx.afterStartLine).toBe(2);
  });

  it("startLine = 0 — before est vide (pas d'index négatif)", () => {
    const ctx = extractDiffContext(FICHIER_10_LIGNES, 0, 1);
    expect(ctx.before).toEqual([]);
  });

  it("fichier vide (string vide) — ne crash pas", () => {
    expect(() => extractDiffContext("", 1, 0)).not.toThrow();
    const ctx = extractDiffContext("", 1, 0);
    expect(ctx.before).toEqual([]);
  });

  it("newLineCount plus grand que le fichier — after est vide", () => {
    const ctx = extractDiffContext(FICHIER_10_LIGNES, 1, 20);
    expect(ctx.after).toEqual([]);
  });
});
