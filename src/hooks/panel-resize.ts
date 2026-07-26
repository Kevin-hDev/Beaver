import type React from "react";

export const PANEL_RESIZING_CLASS = "al-panel-resizing";

export function beginPanelResize(
  event: React.PointerEvent,
  panelSelector: string,
): () => void {
  event.preventDefault();
  const target = event.currentTarget as HTMLElement;
  const panel = target.closest(panelSelector) ?? document.querySelector(panelSelector);

  document.body.classList.add(PANEL_RESIZING_CLASS);
  panel?.classList.add("resizing");

  try {
    target.setPointerCapture(event.pointerId);
  } catch {
    // Pointer capture can be unavailable in tests or old WebViews.
  }

  let cleaned = false;
  return () => {
    if (cleaned) return;
    cleaned = true;
    try {
      if (target.hasPointerCapture?.(event.pointerId)) {
        target.releasePointerCapture(event.pointerId);
      }
    } catch {
      // Cleanup must never leave the global resizing class stuck.
    }
    panel?.classList.remove("resizing");
    document.body.classList.remove(PANEL_RESIZING_CLASS);
  };
}
