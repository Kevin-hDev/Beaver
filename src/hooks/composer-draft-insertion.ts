export interface ComposerSelection {
  anchor: number;
  head: number;
}

function validPosition(text: string, position: number): boolean {
  if (!Number.isSafeInteger(position) || position < 0 || position > text.length) return false;
  if (position === 0 || position === text.length) return true;
  const before = text.charCodeAt(position - 1);
  const after = text.charCodeAt(position);
  return !(before >= 0xd800 && before <= 0xdbff && after >= 0xdc00 && after <= 0xdfff);
}

export function insertAtSelection(
  text: string,
  inserted: string,
  selection: ComposerSelection | null,
): { text: string; cursor: number } {
  const candidate = selection ? Math.min(selection.anchor, selection.head) : text.length;
  const position = selection
    && validPosition(text, selection.anchor)
    && validPosition(text, selection.head)
    && validPosition(text, candidate)
    ? candidate
    : text.length;
  return {
    text: `${text.slice(0, position)}${inserted}${text.slice(position)}`,
    cursor: position + inserted.length,
  };
}
