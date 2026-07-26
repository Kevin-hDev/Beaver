export const USER_MESSAGE_MAX_LINES = 20;
export const USER_MESSAGE_EDIT_MIN_LINES = 2;

const FALLBACK_FONT_SIZE = 14;
const USER_MESSAGE_LINE_HEIGHT_RATIO = 1.55;

export function userMessageHeightForLines(element: Element, lines: number): number {
  const safeLines = Number.isFinite(lines) && lines > 0 ? lines : 1;
  return Math.ceil(readUserMessageLineHeight(element) * safeLines);
}

function readUserMessageLineHeight(element: Element): number {
  const styles = window.getComputedStyle(element);
  const lineHeight = parsePositivePx(styles.lineHeight);
  if (lineHeight) return lineHeight;

  const fontSize = parsePositivePx(styles.fontSize) ?? FALLBACK_FONT_SIZE;
  return fontSize * USER_MESSAGE_LINE_HEIGHT_RATIO;
}

function parsePositivePx(value: string): number | null {
  const normalized = value.trim();
  if (!normalized.endsWith("px")) return null;
  const parsed = Number.parseFloat(normalized);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}
