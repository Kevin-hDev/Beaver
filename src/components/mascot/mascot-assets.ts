import beaverSheet from "@/assets/mascot/cl-go-beaver/master.webp";
import beaverManifest from "@/assets/mascot/cl-go-beaver/manifest.json";
import circuitActionsSheet from "@/assets/mascot/circuit/actions.webp";
import circuitManifest from "@/assets/mascot/circuit/manifest.json";
import circuitStandardSheet from "@/assets/mascot/circuit/standard.webp";
import kovaActionsSheet from "@/assets/mascot/kova/actions.webp";
import kovaManifest from "@/assets/mascot/kova/manifest.json";
import kovaStandardSheet from "@/assets/mascot/kova/standard.webp";
import {
  DEFAULT_MASCOT_ID,
  isMascotId,
  type MascotId,
} from "@/types/mascot";

export const MASCOT_ANIMATION_IDS = [
  "idle", "move-right", "move-left", "wave", "jump", "failed", "waiting",
  "thinking", "explore-book", "look-000-157.5", "look-180-337.5",
  "work-laptop", "success", "celebrate", "grabbed", "held", "dropped",
  "sleeping", "alert",
] as const;

export type MascotAnimationId = typeof MASCOT_ANIMATION_IDS[number];

interface MascotStateSpec {
  id: string;
  sheet?: string;
  row: number;
  startFrame?: number;
  frames: number;
  loop: boolean;
  frameDurationMs?: number;
  loopPauseMs?: number;
  durationsMs?: number[];
}

interface MascotManifest {
  cellWidth: number;
  cellHeight: number;
  states: MascotStateSpec[];
}

interface MascotSheet {
  src: string;
  columns: number;
  rows: number;
}

interface MascotBundle {
  manifest: MascotManifest;
  defaultSheet: string;
  sheets: Record<string, MascotSheet>;
}

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

const MASCOT_BUNDLES: Record<MascotId, MascotBundle> = {
  "cl-go-beaver": {
    manifest: beaverManifest,
    defaultSheet: "master",
    sheets: {
      master: {
        src: beaverSheet,
        columns: beaverManifest.columns,
        rows: beaverManifest.states.length,
      },
    },
  },
  circuit: {
    manifest: circuitManifest,
    defaultSheet: "standard",
    sheets: {
      standard: {
        src: circuitStandardSheet,
        columns: circuitManifest.sheets.standard.columns,
        rows: circuitManifest.sheets.standard.rows,
      },
      actions: {
        src: circuitActionsSheet,
        columns: circuitManifest.sheets.actions.columns,
        rows: circuitManifest.sheets.actions.rows,
      },
    },
  },
  kova: {
    manifest: kovaManifest,
    defaultSheet: "standard",
    sheets: {
      standard: {
        src: kovaStandardSheet,
        columns: kovaManifest.sheets.standard.columns,
        rows: kovaManifest.sheets.standard.rows,
      },
      actions: {
        src: kovaActionsSheet,
        columns: kovaManifest.sheets.actions.columns,
        rows: kovaManifest.sheets.actions.rows,
      },
    },
  },
};

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
    loopPauseMs: state.loopPauseMs,
    durationsMs: state.durationsMs,
  };
}

export function spritePosition(
  frame: number,
  row: number,
  columns = beaverManifest.columns,
  rows = beaverManifest.states.length,
): string {
  const x = columns <= 1 ? 0 : frame / (columns - 1) * 100;
  const y = rows <= 1 ? 0 : row / (rows - 1) * 100;
  return `${x}% ${y}%`;
}
