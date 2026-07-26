const DEFAULT_REM_BASE = 16;

export function cssLengthToPx(value: string, remBase: number): number | null {
  const trimmed = value.trim();
  const amount = Number.parseFloat(trimmed);
  if (!Number.isFinite(amount)) return null;
  if (trimmed.endsWith("rem")) return amount * remBase;
  if (trimmed.endsWith("px") || /^[0-9.]+$/.test(trimmed)) return amount;
  return null;
}

export function getListMinWidthPx(listEl: HTMLElement, fallback: number): number {
  const rootStyle = getComputedStyle(document.documentElement);
  const remBase = cssLengthToPx(rootStyle.fontSize, DEFAULT_REM_BASE) ?? DEFAULT_REM_BASE;
  const listWidth = getComputedStyle(listEl).getPropertyValue("--list-width");
  return cssLengthToPx(listWidth, remBase) ?? fallback;
}

export function nextListPanelWidth(startWidth: number, minWidth: number, delta: number): number | null {
  const nextWidth = Math.max(minWidth, startWidth + delta);
  return nextWidth > minWidth ? nextWidth : null;
}
