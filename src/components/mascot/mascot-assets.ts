import masterSheet from "@/assets/mascot/cl-go-beaver/master.webp";
import manifest from "@/assets/mascot/cl-go-beaver/manifest.json";

export const MASCOT_ANIMATION_IDS = [
  "idle", "move-right", "move-left", "wave", "jump", "failed", "waiting",
  "thinking", "explore-book", "look-000-157.5", "look-180-337.5",
  "work-laptop", "success", "celebrate", "grabbed", "held", "dropped",
  "sleeping", "alert",
] as const;

export type MascotAnimationId = typeof MASCOT_ANIMATION_IDS[number];

export interface MascotAnimationDefinition {
  id: MascotAnimationId;
  row: number;
  frames: number;
  loop: boolean;
  frameDurationMs?: number;
  loopPauseMs?: number;
  durationsMs?: number[];
}

export const MASCOT_SHEET = masterSheet;
export const MASCOT_COLUMNS = manifest.columns;
export const MASCOT_ROWS = manifest.states.length;
export const MASCOT_FRAME_RATIO = manifest.cellWidth / manifest.cellHeight;
export const DEFAULT_FRAME_DURATION_MS = 180;

export function getMascotAnimation(id: MascotAnimationId): MascotAnimationDefinition {
  const state = manifest.states.find((candidate) => candidate.id === id)
    ?? manifest.states[0];
  return {
    id,
    row: state.row,
    frames: Math.max(1, Math.min(MASCOT_COLUMNS, state.frames)),
    loop: state.loop,
    frameDurationMs: "frameDurationMs" in state ? state.frameDurationMs : undefined,
    loopPauseMs: "loopPauseMs" in state ? state.loopPauseMs : undefined,
    durationsMs: "durationsMs" in state ? state.durationsMs : undefined,
  };
}

export function spritePosition(frame: number, row: number): string {
  const x = MASCOT_COLUMNS <= 1 ? 0 : frame / (MASCOT_COLUMNS - 1) * 100;
  const y = MASCOT_ROWS <= 1 ? 0 : row / (MASCOT_ROWS - 1) * 100;
  return `${x}% ${y}%`;
}
