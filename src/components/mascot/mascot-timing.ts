export const MASCOT_LOOP_PAUSE_MS: Readonly<Record<string, number>> = {
  idle: 4500,
  waiting: 3500,
  thinking: 3000,
  "explore-book": 3000,
  "work-laptop": 2500,
};

export function mascotLoopPauseMs(
  animationId: string,
  fallback?: number,
): number | undefined {
  return MASCOT_LOOP_PAUSE_MS[animationId] ?? fallback;
}
