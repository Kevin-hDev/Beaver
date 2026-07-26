import { afterEach, describe, expect, it, vi } from "vitest";
import { beginPanelResize, PANEL_RESIZING_CLASS } from "../panel-resize";
import type React from "react";

describe("beginPanelResize", () => {
  afterEach(() => {
    document.body.innerHTML = "";
    document.body.className = "";
    vi.restoreAllMocks();
  });

  it("pose la classe globale immédiatement et nettoie sur stop", () => {
    document.body.innerHTML = `
      <aside class="ft-panel">
        <div class="ft-resize"></div>
      </aside>
    `;
    const handle = document.querySelector(".ft-resize") as HTMLElement;
    const panel = document.querySelector(".ft-panel") as HTMLElement;
    const preventDefault = vi.fn();
    const setPointerCapture = vi.fn();
    const hasPointerCapture = vi.fn(() => true);
    const releasePointerCapture = vi.fn();
    handle.setPointerCapture = setPointerCapture;
    handle.hasPointerCapture = hasPointerCapture;
    handle.releasePointerCapture = releasePointerCapture;

    const stop = beginPanelResize(
      {
        preventDefault,
        currentTarget: handle,
        pointerId: 7,
      } as unknown as React.PointerEvent,
      ".ft-panel",
    );

    expect(preventDefault).toHaveBeenCalled();
    expect(setPointerCapture).toHaveBeenCalledWith(7);
    expect(document.body.classList.contains(PANEL_RESIZING_CLASS)).toBe(true);
    expect(panel.classList.contains("resizing")).toBe(true);

    stop();

    expect(releasePointerCapture).toHaveBeenCalledWith(7);
    expect(document.body.classList.contains(PANEL_RESIZING_CLASS)).toBe(false);
    expect(panel.classList.contains("resizing")).toBe(false);
  });
});
