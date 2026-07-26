import { afterEach, describe, expect, it } from "vitest";
import {
  FILE_TREE_MAX_WIDTH,
  FILE_TREE_MIN_WIDTH,
  clampFileTreeWidthForContainer,
  measureFileTreeLayout,
} from "../file-tree-layout";

function setWidth(element: Element, width: number) {
  element.getBoundingClientRect = () => ({
    width,
    height: 0,
    top: 0,
    right: width,
    bottom: 0,
    left: 0,
    x: 0,
    y: 0,
    toJSON: () => ({}),
  });
}

describe("file tree layout", () => {
  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("borne l'arbre avec la largeur prise par la preview", () => {
    expect(clampFileTreeWidthForContainer(500, 1000, 360)).toBe(280);
  });

  it("garde la largeur minimale quand il y a assez de place", () => {
    expect(clampFileTreeWidthForContainer(120, 1200, 360)).toBe(FILE_TREE_MIN_WIDTH);
  });

  it("ne descend pas vers un fallback illisible quand l'espace est serre", () => {
    expect(clampFileTreeWidthForContainer(80, 610, 120)).toBe(FILE_TREE_MIN_WIDTH);
  });

  it("utilise le minimum dynamique du chat en mode compact", () => {
    expect(clampFileTreeWidthForContainer(240, 400, 250, 0)).toBe(150);
  });

  it("respecte la largeur maximale statique", () => {
    expect(clampFileTreeWidthForContainer(900, 1600, 360)).toBe(FILE_TREE_MAX_WIDTH);
  });

  it("mesure les panneaux voisins sauf le chat et l'arbre courant", () => {
    document.body.innerHTML = `
      <div class="agent-detail-with-preview">
        <main class="agent-detail-chat"></main>
        <aside class="asp-panel open"></aside>
        <aside class="ft-panel open"></aside>
      </div>
    `;
    const container = document.querySelector(".agent-detail-with-preview")!;
    const preview = document.querySelector(".asp-panel")!;
    const tree = document.querySelector(".ft-panel")!;
    setWidth(container, 1200);
    setWidth(preview, 420);
    setWidth(tree, 240);

    expect(measureFileTreeLayout(tree)).toMatchObject({
      container,
      containerWidth: 1200,
      reservedWidth: 420,
    });
  });
});
