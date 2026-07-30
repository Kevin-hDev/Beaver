export interface MascotStateSpec {
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

export interface MascotManifest {
  cellWidth: number;
  cellHeight: number;
  states: MascotStateSpec[];
}

export interface MascotSheet {
  src: string;
  columns: number;
  rows: number;
}

export interface MascotBundle {
  manifest: MascotManifest;
  defaultSheet: string;
  sheets: Record<string, MascotSheet>;
}
