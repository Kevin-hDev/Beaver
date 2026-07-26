const MENU_GAP_PX = 4;
const VIEWPORT_INSET_PX = 8;

interface ViewportSize {
  width: number;
  height: number;
}

export interface ModeMenuPosition {
  top: number;
  right: number;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), Math.max(minimum, maximum));
}

function overlaps(viewportWidth: number, top: number, right: number, menu: DOMRect, surface: DOMRect) {
  const menuLeft = viewportWidth - right - menu.width;
  return menuLeft < surface.right && menuLeft + menu.width > surface.left &&
    top < surface.bottom && top + menu.height > surface.top;
}

export function resolveModeMenuPosition(
  anchor: DOMRect,
  menu: DOMRect,
  surface: DOMRect | null,
  viewport: ViewportSize,
): ModeMenuPosition {
  const maxTop = viewport.height - menu.height - VIEWPORT_INSET_PX;
  const maxRight = viewport.width - menu.width - VIEWPORT_INSET_PX;
  const defaultTop = clamp(anchor.bottom + MENU_GAP_PX, VIEWPORT_INSET_PX, maxTop);
  const defaultRight = clamp(viewport.width - anchor.right, VIEWPORT_INSET_PX, maxRight);

  if (!surface || !overlaps(viewport.width, defaultTop, defaultRight, menu, surface)) {
    return { top: defaultTop, right: defaultRight };
  }

  const aboveSurface = surface.top - menu.height - MENU_GAP_PX;
  if (aboveSurface >= VIEWPORT_INSET_PX) {
    return {
      top: aboveSurface,
      right: clamp(viewport.width - anchor.left + MENU_GAP_PX, VIEWPORT_INSET_PX, maxRight),
    };
  }

  const leftOfSurface = surface.left - menu.width - MENU_GAP_PX;
  if (leftOfSurface >= VIEWPORT_INSET_PX) {
    return {
      top: defaultTop,
      right: viewport.width - leftOfSurface - menu.width,
    };
  }

  return { top: VIEWPORT_INSET_PX, right: defaultRight };
}
