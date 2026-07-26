import { invoke } from "@tauri-apps/api/core";

export const MASCOT_SIZE_MIN = 70;
export const MASCOT_SIZE_MAX = 140;
export const DEFAULT_MASCOT_SETTINGS: MascotSettings = {
  enabled: false,
  mascot_id: "cl-go-beaver",
  size_percent: 100,
  position: null,
};

export type MascotRuntimeAnimation =
  | "idle"
  | "thinking"
  | "explore-book"
  | "work-laptop"
  | "waiting"
  | "success"
  | "failed"
  | "alert";

export interface MascotSettings {
  enabled: boolean;
  mascot_id: string;
  size_percent: number;
  position: { x: number; y: number } | null;
}

export interface MascotState {
  animation: MascotRuntimeAnimation;
  revision: number;
}

export type MascotSettingsPatch = Partial<Pick<MascotSettings,
  "enabled" | "mascot_id" | "size_percent"
>>;

const VALID_RUNTIME_ANIMATIONS = new Set<MascotRuntimeAnimation>([
  "idle", "thinking", "explore-book", "work-laptop", "waiting", "success", "failed", "alert",
]);

export async function getMascotSettings(): Promise<MascotSettings> {
  return normalizeMascotSettings(await invoke<unknown>("get_mascot_settings"));
}

export async function patchMascotSettings(
  patch: MascotSettingsPatch,
): Promise<MascotSettings> {
  const safePatch: MascotSettingsPatch = {};
  if (typeof patch.enabled === "boolean") safePatch.enabled = patch.enabled;
  if (patch.mascot_id === "cl-go-beaver") safePatch.mascot_id = patch.mascot_id;
  if (typeof patch.size_percent === "number") {
    safePatch.size_percent = clampSize(patch.size_percent);
  }
  return normalizeMascotSettings(
    await invoke<unknown>("patch_mascot_settings", { patch: safePatch }),
  );
}

export async function getMascotState(): Promise<MascotState> {
  return normalizeMascotState(await invoke<unknown>("get_mascot_state"));
}

export async function saveMascotPosition(x: number, y: number): Promise<void> {
  if (!isSafePosition(x) || !isSafePosition(y)) return;
  await invoke("save_mascot_position", { x: Math.round(x), y: Math.round(y) });
}

export function normalizeMascotSettings(value: unknown): MascotSettings {
  if (!value || typeof value !== "object") return DEFAULT_MASCOT_SETTINGS;
  const record = value as Record<string, unknown>;
  const rawPosition = record.position;
  const position = rawPosition && typeof rawPosition === "object"
    ? normalizePosition(rawPosition as Record<string, unknown>)
    : null;
  return {
    enabled: record.enabled === true,
    mascot_id: record.mascot_id === "cl-go-beaver" ? record.mascot_id : "cl-go-beaver",
    size_percent: clampSize(Number(record.size_percent)),
    position,
  };
}

export function normalizeMascotState(value: unknown): MascotState {
  if (!value || typeof value !== "object") return { animation: "idle", revision: 0 };
  const record = value as Record<string, unknown>;
  const animation = VALID_RUNTIME_ANIMATIONS.has(record.animation as MascotRuntimeAnimation)
    ? record.animation as MascotRuntimeAnimation
    : "idle";
  const revision = Number.isSafeInteger(record.revision) && Number(record.revision) >= 0
    ? Number(record.revision)
    : 0;
  return { animation, revision };
}

function clampSize(value: number): number {
  if (!Number.isFinite(value)) return DEFAULT_MASCOT_SETTINGS.size_percent;
  return Math.round(Math.min(MASCOT_SIZE_MAX, Math.max(MASCOT_SIZE_MIN, value)));
}

function normalizePosition(record: Record<string, unknown>) {
  const x = Number(record.x);
  const y = Number(record.y);
  return isSafePosition(x) && isSafePosition(y) ? { x, y } : null;
}

function isSafePosition(value: number): boolean {
  return Number.isSafeInteger(value) && Math.abs(value) <= 100_000;
}
