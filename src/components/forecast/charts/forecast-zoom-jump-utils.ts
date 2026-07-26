// Pure helpers for the zoom jump bars overlay (no React / ECharts imports).

export function zoomJumpVisible(span: number): boolean {
  return Number.isFinite(span) && span > 0 && span <= 50;
}

export function zoomJumpBarCount(span: number): number {
  const level = 100 - span;
  if (level >= 90) return 5;
  if (level >= 75) return 4;
  if (level >= 50) return 3;
  return 0;
}

// Start position (percent) for bar `index`, keeping the current span.
export function zoomJumpTarget(index: number, count: number, span: number): number {
  if (count <= 1) return 0;
  const maxStart = Math.max(0, 100 - span);
  return (index / (count - 1)) * maxStart;
}

export function zoomJumpCurrentIndex(start: number, span: number, count: number): number {
  const maxStart = Math.max(0, 100 - span);
  if (count <= 1 || maxStart <= 0) return 0;
  const ratio = Math.min(1, Math.max(0, start / maxStart));
  return Math.round(ratio * (count - 1));
}
