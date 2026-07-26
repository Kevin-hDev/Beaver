import { describe, expect, it } from "vitest";
import {
  CHAT_COMPACT_MIN_WIDTH,
  computeAgentPanelLayout,
  shouldAutoHideSidebarForAgentPanels,
} from "../agent-panel-layout-solver";
import { CHAT_MIN_WIDTH, FILE_PREVIEW_MIN_WIDTH } from "../file-preview-storage";
import { FILE_TREE_DEFAULT_WIDTH, FILE_TREE_MIN_WIDTH } from "../file-tree-layout";

describe("agent panel layout solver", () => {
  it("garde les largeurs normales quand la fenetre est large", () => {
    const layout = computeAgentPanelLayout({
      containerWidth: 1200,
      chatTargetWidth: CHAT_MIN_WIDTH,
      previewOpen: true,
      previewDesiredWidth: 360,
      fileTreeOpen: true,
      fileTreeDesiredWidth: FILE_TREE_DEFAULT_WIDTH,
    });

    expect(layout).toMatchObject({
      chatMinWidth: CHAT_MIN_WIDTH,
      previewWidth: 360,
      fileTreeWidth: FILE_TREE_DEFAULT_WIDTH,
      panelsTight: false,
    });
    expect(layout.totalReservedWidth).toBeLessThanOrEqual(1200);
  });

  it("demande l'auto-hide quand preview et arbre sont trop serres sans override", () => {
    expect(shouldAutoHideSidebarForAgentPanels(760, true, true)).toBe(true);
    expect(shouldAutoHideSidebarForAgentPanels(780, true, true)).toBe(false);
  });

  it("compacte le chat apres reouverture manuelle sans masquer les panels", () => {
    const layout = computeAgentPanelLayout({
      containerWidth: 700,
      chatTargetWidth: CHAT_COMPACT_MIN_WIDTH,
      previewOpen: true,
      previewDesiredWidth: 360,
      fileTreeOpen: true,
      fileTreeDesiredWidth: FILE_TREE_DEFAULT_WIDTH,
    });

    expect(layout.chatMinWidth).toBe(CHAT_COMPACT_MIN_WIDTH);
    expect(layout.previewWidth).toBeGreaterThanOrEqual(FILE_PREVIEW_MIN_WIDTH);
    expect(layout.fileTreeWidth).toBeGreaterThanOrEqual(FILE_TREE_MIN_WIDTH);
    expect(layout.totalReservedWidth).toBeLessThanOrEqual(700);
  });

  it("reserve l'arbre quand la preview est tiree au maximum", () => {
    const layout = computeAgentPanelLayout({
      containerWidth: 900,
      chatTargetWidth: CHAT_MIN_WIDTH,
      previewOpen: true,
      previewDesiredWidth: 800,
      fileTreeOpen: true,
      fileTreeDesiredWidth: FILE_TREE_DEFAULT_WIDTH,
    });

    expect(layout.fileTreeWidth).toBeGreaterThanOrEqual(FILE_TREE_MIN_WIDTH);
    expect(layout.previewWidth).toBe(300);
    expect(layout.totalReservedWidth).toBe(900);
  });

  it("ne descend pas l'arbre a 120px quand 160px sont disponibles", () => {
    const layout = computeAgentPanelLayout({
      containerWidth: 770,
      chatTargetWidth: CHAT_MIN_WIDTH,
      previewOpen: true,
      previewDesiredWidth: FILE_PREVIEW_MIN_WIDTH,
      fileTreeOpen: true,
      fileTreeDesiredWidth: 120,
    });

    expect(layout.fileTreeWidth).toBe(FILE_TREE_MIN_WIDTH);
  });

  it("ne depasse jamais l'espace disponible meme quand tout est impossible", () => {
    const layout = computeAgentPanelLayout({
      containerWidth: 350,
      chatTargetWidth: CHAT_COMPACT_MIN_WIDTH,
      previewOpen: true,
      previewDesiredWidth: 360,
      fileTreeOpen: true,
      fileTreeDesiredWidth: FILE_TREE_DEFAULT_WIDTH,
    });

    expect(layout.fileTreeWidth).toBe(FILE_TREE_MIN_WIDTH);
    expect(layout.totalReservedWidth).toBeLessThanOrEqual(350);
  });
});
