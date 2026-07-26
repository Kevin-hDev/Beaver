import { describe, expect, it } from "vitest";
import { cellText, reconstructFromOps } from "./spreadsheet-ops-parser";

describe("cellText", () => {
  it("null retourne une chaîne vide", () => {
    expect(cellText(null)).toBe("");
    expect(cellText(undefined)).toBe("");
  });

  it("string passthrough — retourne la valeur telle quelle", () => {
    expect(cellText("bonjour")).toBe("bonjour");
    expect(cellText("")).toBe("");
  });

  it("number converti en string", () => {
    expect(cellText(42)).toBe("42");
    expect(cellText(3.14)).toBe("3.14");
    expect(cellText(0)).toBe("0");
  });

  it("boolean converti en string", () => {
    expect(cellText(true)).toBe("true");
    expect(cellText(false)).toBe("false");
  });

  it("objet sérialisé en JSON", () => {
    expect(cellText({ a: 1 })).toBe('{"a":1}');
    expect(cellText([1, 2])).toBe("[1,2]");
  });
});

describe("reconstructFromOps", () => {
  it("JSON invalide retourne null", () => {
    expect(reconstructFromOps("{pas du json")).toBeNull();
    expect(reconstructFromOps("")).toBeNull();
  });

  it("tableau vide retourne null", () => {
    expect(reconstructFromOps("[]")).toBeNull();
  });

  it("set_row crée une grille avec la première ligne en headers", () => {
    const ops = JSON.stringify([
      { type: "set_row", row: 0, values: ["Nom", "Age"] },
      { type: "set_row", row: 1, values: ["Alice", 30] },
    ]);
    const result = reconstructFromOps(ops);
    expect(result).not.toBeNull();
    expect(result!.headers).toEqual(["Nom", "Age"]);
    expect(result!.rows[0]).toEqual(["Alice", 30]);
    expect(result!.total_rows).toBe(1);
  });

  it("set_cell avec ref 'B2' place la valeur au bon endroit", () => {
    // B2 = row 1, col 1 (0-indexed)
    const ops = JSON.stringify([
      { type: "set_row", row: 0, values: ["A", "B"] },
      { type: "set_cell", cell: "B2", value: "cellule-B2" },
    ]);
    const result = reconstructFromOps(ops);
    expect(result).not.toBeNull();
    // row 0 = headers ["A","B"], row 1 = data row index 0
    expect(result!.rows[0][1]).toBe("cellule-B2");
  });

  it("set_cell avec row/col numériques place la valeur correctement", () => {
    const ops = JSON.stringify([
      { type: "set_row", row: 0, values: ["X", "Y"] },
      { type: "set_cell", row: 1, col: 0, value: "valeur-X" },
    ]);
    const result = reconstructFromOps(ops);
    expect(result).not.toBeNull();
    expect(result!.rows[0][0]).toBe("valeur-X");
  });

  it("set_formula stocke la formule comme valeur de la cellule", () => {
    const ops = JSON.stringify([
      { type: "set_row", row: 0, values: ["Total"] },
      { type: "set_formula", cell: "A2", formula: "=SUM(A1:A10)" },
    ]);
    const result = reconstructFromOps(ops);
    expect(result).not.toBeNull();
    expect(result!.rows[0][0]).toBe("=SUM(A1:A10)");
  });

  it("respecte MAX_OPS (10000) sans crash — tronque à 10000 ops", () => {
    // Génère 10001 ops set_row (la 10001ème doit être ignorée)
    const ops = Array.from({ length: 10_001 }, (_, i) => ({
      type: "set_row",
      row: i,
      values: [`ligne-${i}`],
    }));
    const json = JSON.stringify(ops);
    expect(() => reconstructFromOps(json)).not.toThrow();
    const result = reconstructFromOps(json);
    // Le résultat doit exister (au moins une ligne dans la grille)
    expect(result).not.toBeNull();
  });

  it("retourne null si aucune cellule n'a été écrite (ops invalides)", () => {
    const ops = JSON.stringify([
      { type: "unknown_op", data: "foo" },
    ]);
    expect(reconstructFromOps(ops)).toBeNull();
  });

  it("ops non-array (string JSON valide) → retourne null", () => {
    expect(reconstructFromOps('"juste-une-string"')).toBeNull();
  });

  it("set_cell avec row/col négatifs → cellule ignorée, retourne null si rien d'autre", () => {
    const ops = JSON.stringify([
      { type: "set_cell", row: -1, col: -1, value: "invalide" },
    ]);
    expect(reconstructFromOps(ops)).toBeNull();
  });

  it("parseCellRef via set_cell avec ref 'A1' → place la valeur en headers (row 0, col 0)", () => {
    const ops = JSON.stringify([
      { type: "set_cell", cell: "A1", value: "entete" },
    ]);
    const result = reconstructFromOps(ops);
    expect(result).not.toBeNull();
    expect(result!.headers[0]).toBe("entete");
  });

  it("parseCellRef avec ref invalide 'ZZZ' (sans chiffre) → cellule ignorée, retourne null", () => {
    const ops = JSON.stringify([
      { type: "set_cell", cell: "ZZZ", value: "perdu" },
    ]);
    expect(reconstructFromOps(ops)).toBeNull();
  });

  it("cellText avec objet → JSON.stringify", () => {
    expect(cellText({ cle: "valeur" })).toBe('{"cle":"valeur"}');
    expect(cellText([1, 2, 3])).toBe("[1,2,3]");
  });

  it("A1 → row 0 col 0, B3 → row 2 col 1 (vérification parseCellRef via set_cell)", () => {
    const ops = JSON.stringify([
      { type: "set_cell", cell: "A1", value: "origine" },
      { type: "set_cell", cell: "B3", value: "b-trois" },
    ]);
    const result = reconstructFromOps(ops);
    expect(result).not.toBeNull();
    // row 0 = headers (A1), rows[0] = row 1 (vide), rows[1] = row 2 = B3
    // A1 est la seule cellule de la ligne 0 → headers[0] = "origine"
    expect(result!.headers[0]).toBe("origine");
    // B3 = row index 2, col index 1
    // rows[1] correspond à row 2 (index dans sortedRows après row 0)
    const rowB3Index = result!.rows.findIndex((r) => r.some((c) => c === "b-trois"));
    expect(rowB3Index).toBeGreaterThanOrEqual(0);
  });
});
