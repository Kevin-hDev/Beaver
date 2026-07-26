import { describe, expect, it } from "vitest";
import {
  CHAT_MIN_WIDTH,
  FILE_PREVIEW_MIN_WIDTH,
  clampFilePreviewWidthForContainer,
} from "../file-preview-storage";

describe("clampFilePreviewWidthForContainer", () => {
  it("garde au moins 360px pour le chat quand le panel grandit", () => {
    expect(clampFilePreviewWidthForContainer(900, 1000)).toBe(1000 - CHAT_MIN_WIDTH);
  });

  it("garde la largeur minimale du panel quand il y a assez de place", () => {
    expect(clampFilePreviewWidthForContainer(120, 1000)).toBe(FILE_PREVIEW_MIN_WIDTH);
  });

  it("ouvre par défaut avec une largeur plus confortable que le minimum", async () => {
    const storage = await import("../file-preview-storage");
    expect(storage.FILE_PREVIEW_DEFAULT_WIDTH).toBe(360);
    expect(storage.FILE_PREVIEW_MIN_WIDTH).toBe(250);
  });

  it("tient compte de la largeur forecast ajoutée au panel", () => {
    expect(clampFilePreviewWidthForContainer(900, 1000, 100)).toBe(1000 - CHAT_MIN_WIDTH - 100);
  });

  it("garde le minimum preview et laisse le chat se compacter si besoin", () => {
    expect(clampFilePreviewWidthForContainer(360, 560)).toBe(FILE_PREVIEW_MIN_WIDTH);
  });

  it("laisse la preview garder sa largeur quand le chat compact accepte 0px", () => {
    expect(clampFilePreviewWidthForContainer(360, 560, 0, 0)).toBe(360);
  });
});
