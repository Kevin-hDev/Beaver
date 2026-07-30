import {
  DEFAULT_MASCOT_ID,
  isMascotId,
  type MascotId,
} from "@/types/mascot";
import type { MascotSheet } from "./mascot-bundle-types";
import { MASCOT_BUNDLES } from "./mascot-bundles";
import { mascotLoopPauseMs } from "./mascot-timing";

export const MASCOT_ANIMATION_IDS = [
  "idle", "move-right", "move-left", "wave", "jump", "failed", "waiting",
  "thinking", "explore-book", "look-000-157.5", "look-180-337.5",
  "work-laptop", "success", "celebrate", "grabbed", "held", "dropped",
  "sleeping", "alert",
] as const;

export type MascotAnimationId = typeof MASCOT_ANIMATION_IDS[number];

export interface MascotAnimationDefinition extends MascotSheet {
  id: MascotAnimationId;
  row: number;
  startFrame: number;
  frames: number;
  loop: boolean;
  frameRatio: number;
  frameDurationMs?: number;
  loopPauseMs?: number;
  durationsMs?: number[];
}

export const DEFAULT_FRAME_DURATION_MS = 180;

export function getMascotAnimation(
  id: MascotAnimationId,
  mascotId: MascotId = DEFAULT_MASCOT_ID,
): MascotAnimationDefinition {
  const bundle = MASCOT_BUNDLES[isMascotId(mascotId) ? mascotId : DEFAULT_MASCOT_ID];
  const state = bundle.manifest.states.find((candidate) => candidate.id === id)
    ?? bundle.manifest.states[0];
  const sheet = bundle.sheets[state.sheet ?? bundle.defaultSheet]
    ?? bundle.sheets[bundle.defaultSheet];
  const startFrame = Math.max(
    0,
    Math.min(sheet.columns - 1, state.startFrame ?? 0),
  );

  return {
    id,
    src: sheet.src,
    columns: sheet.columns,
    rows: sheet.rows,
    row: state.row,
    startFrame,
    frames: Math.max(1, Math.min(sheet.columns - startFrame, state.frames)),
    loop: state.loop,
    frameRatio: bundle.manifest.cellWidth / bundle.manifest.cellHeight,
    frameDurationMs: state.frameDurationMs,
    loopPauseMs: mascotLoopPauseMs(state.id, state.loopPauseMs),
    durationsMs: state.durationsMs,
  };
}

export function spritePosition(
  frame: number,
  row: number,
  columns = 8,
  rows = 19,
): string {
  const x = columns <= 1 ? 0 : frame / (columns - 1) * 100;
  const y = rows <= 1 ? 0 : row / (rows - 1) * 100;
  return `${x}% ${y}%`;
}
