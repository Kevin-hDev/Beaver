import { describe, expect, it, vi } from "vitest";
import { measureBrowserSurface } from "../browser-surface-measure";

function rect(x: number, y: number, width: number, height: number): DOMRect {
  return {
    x,
    y,
    width,
    height,
    top: y,
    right: x + width,
    bottom: y + height,
    left: x,
    toJSON: () => ({}),
  };
}

describe("measureBrowserSurface", () => {
  it("utilise toute la largeur du panneau sans créer de gouttière", () => {
    const panel = document.createElement("aside");
    panel.className = "asp-panel";
    const handle = document.createElement("div");
    handle.className = "asp-resize";
    const surface = document.createElement("div");
    panel.append(handle, surface);
    vi.spyOn(surface, "getBoundingClientRect").mockReturnValue(rect(420, 180, 600, 500));
    vi.spyOn(handle, "getBoundingClientRect").mockReturnValue(rect(420, 0, 6, 680));

    expect(measureBrowserSurface(surface, 1)).toMatchObject({
      x: 420,
      y: 180,
      width: 600,
      height: 500,
    });
  });

  it("utilise toute la largeur lorsque le panneau est en plein écran", () => {
    const panel = document.createElement("aside");
    panel.className = "asp-panel fullscreen";
    const handle = document.createElement("div");
    handle.className = "asp-resize";
    const surface = document.createElement("div");
    panel.append(handle, surface);
    vi.spyOn(surface, "getBoundingClientRect").mockReturnValue(rect(240, 112, 912, 656));
    vi.spyOn(handle, "getBoundingClientRect").mockReturnValue(rect(240, 0, 6, 768));

    expect(measureBrowserSurface(surface, 1)).toMatchObject({ x: 240, width: 912 });
  });
});
