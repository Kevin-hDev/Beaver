import { describe, expect, it } from "vitest";
import {
  moveId,
  sameOrder,
  slotGap,
  slotOffsets,
  targetIndex,
  type DragSlot,
} from "./drag-reorder-geometry";

/* Trois cases de 40, séparées de 4, comme les projets de la barre latérale.
   Milieux : 20, 64, 108. */
const uniform: DragSlot[] = [
  { id: "a", start: 0, size: 40 },
  { id: "b", start: 44, size: 40 },
  { id: "c", start: 88, size: 40 },
];

/* Cases de tailles très différentes, comme un projet déplié à côté d'un projet
   replié. Milieux : 15, 130, 265. */
const uneven: DragSlot[] = [
  { id: "a", start: 0, size: 30 },
  { id: "b", start: 30, size: 200 },
  { id: "c", start: 230, size: 70 },
];

describe("slotGap", () => {
  it("relève l'écart entre deux cases voisines", () => {
    expect(slotGap(uniform)).toBe(4);
  });

  it("vaut zéro quand les cases se touchent", () => {
    expect(slotGap(uneven)).toBe(0);
  });

  it("vaut zéro pour une liste d'une seule case", () => {
    expect(slotGap([uniform[0]])).toBe(0);
  });
});

describe("targetIndex", () => {
  /* Le bornage du déplacement aux limites de la liste rendait ces deux places
     inatteignables : la case tenue s'arrêtait pile là où son milieu rejoignait
     celui du voisin, sans jamais le dépasser. */
  it("atteint la dernière place quand le geste va au-delà de la liste", () => {
    expect(targetIndex(uniform, 0, 999)).toBe(2);
  });

  it("atteint la première place quand le geste remonte au-delà de la liste", () => {
    expect(targetIndex(uniform, 2, -999)).toBe(0);
  });

  it("échange les deux seules cases d'une liste de deux", () => {
    const pair = uniform.slice(0, 2);
    expect(targetIndex(pair, 0, 999)).toBe(1);
    expect(targetIndex(pair, 1, -999)).toBe(0);
  });

  it("ne change rien tant que le milieu du voisin n'est pas franchi", () => {
    /* La case « a » est centrée sur 20 ; le milieu de « b » est à 64. */
    expect(targetIndex(uniform, 0, 43)).toBe(0);
  });

  it("vise le voisin dès que son milieu est franchi", () => {
    expect(targetIndex(uniform, 0, 45)).toBe(1);
  });

  it("vise vers le haut avec la même règle", () => {
    expect(targetIndex(uniform, 2, -43)).toBe(2);
    expect(targetIndex(uniform, 2, -45)).toBe(1);
  });

  it("traverse deux cases d'un coup quand le geste est assez long", () => {
    expect(targetIndex(uniform, 0, 90)).toBe(2);
  });

  it("tient compte de la taille réelle de chaque case", () => {
    /* « c » est centrée sur 265 ; le milieu de « b » est à 130 : il faut donc
       remonter de 135, quelle que soit la hauteur de « c ». */
    expect(targetIndex(uneven, 2, -134)).toBe(2);
    expect(targetIndex(uneven, 2, -136)).toBe(1);
  });
});

describe("slotOffsets", () => {
  it("fait suivre le curseur à la case tenue", () => {
    expect(slotOffsets(uniform, 0, 0, 17).get("a")).toBe(17);
  });

  it("laisse les voisins en place tant que la cible ne change pas", () => {
    const offsets = slotOffsets(uniform, 0, 0, 17);
    expect(offsets.get("b")).toBeUndefined();
    expect(offsets.get("c")).toBeUndefined();
  });

  it("remonte les cases franchies de la place libérée, écart compris", () => {
    const offsets = slotOffsets(uniform, 0, 2, 90);
    expect(offsets.get("b")).toBe(-44);
    expect(offsets.get("c")).toBe(-44);
  });

  it("descend les cases franchies dans l'autre sens", () => {
    const offsets = slotOffsets(uniform, 2, 0, -90);
    expect(offsets.get("a")).toBe(44);
    expect(offsets.get("b")).toBe(44);
  });

  it("décale les voisins de la taille de la case tenue, pas de la leur", () => {
    const offsets = slotOffsets(uneven, 1, 0, -136);
    expect(offsets.get("a")).toBe(200);
  });
});

describe("moveId", () => {
  it("déplace un identifiant vers le bas", () => {
    expect(moveId(["a", "b", "c"], 0, 2)).toEqual(["b", "c", "a"]);
  });

  it("déplace un identifiant vers le haut", () => {
    expect(moveId(["a", "b", "c"], 2, 0)).toEqual(["c", "a", "b"]);
  });
});

describe("sameOrder", () => {
  it("distingue deux ordres différents des mêmes identifiants", () => {
    expect(sameOrder(["a", "b"], ["b", "a"])).toBe(false);
    expect(sameOrder(["a", "b"], ["a", "b"])).toBe(true);
  });

  it("distingue deux listes de longueurs différentes", () => {
    expect(sameOrder(["a"], ["a", "b"])).toBe(false);
  });
});
