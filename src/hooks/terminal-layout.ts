export const TERMINAL_DEFAULT_HEIGHT = 120;
export const TERMINAL_MIN_HEIGHT = 80;
export const TERMINAL_MAX_VIEWPORT_RATIO = 0.5;

/** Une valeur non numérique choisit le minimum sûr au lieu de propager NaN au style. */
export function clampTerminalHeight(requested: number, maximum: number): number {
  if (Number.isNaN(requested)) return TERMINAL_MIN_HEIGHT;
  return Math.max(TERMINAL_MIN_HEIGHT, Math.min(requested, maximum));
}
