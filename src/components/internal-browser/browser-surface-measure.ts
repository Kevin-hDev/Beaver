import {
  MAX_BROWSER_SURFACE_EDGE,
  type BrowserSurfaceBounds,
} from "./browser-types";

export function measureBrowserSurface(
  element: HTMLDivElement,
  generation: number,
): BrowserSurfaceBounds | null {
  const rect = element.getBoundingClientRect();
  const x = Math.round(rect.x);
  const y = Math.round(rect.y);
  const width = Math.round(rect.width);
  const height = Math.round(rect.height);
  if (
    x < 0 || y < 0 || width <= 0 || height <= 0 ||
    x > MAX_BROWSER_SURFACE_EDGE || y > MAX_BROWSER_SURFACE_EDGE ||
    width > MAX_BROWSER_SURFACE_EDGE || height > MAX_BROWSER_SURFACE_EDGE
  ) return null;
  return { x, y, width, height, visible: true, generation };
}

export function sameBrowserBounds(
  left: BrowserSurfaceBounds | null,
  right: BrowserSurfaceBounds,
): boolean {
  return Boolean(
    left?.visible === right.visible && left.x === right.x && left.y === right.y &&
    left.width === right.width && left.height === right.height,
  );
}
