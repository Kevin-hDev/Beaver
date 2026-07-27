export const MASCOT_IDS = ["cl-go-beaver", "circuit", "kova"] as const;

export type MascotId = typeof MASCOT_IDS[number];

export const DEFAULT_MASCOT_ID: MascotId = "cl-go-beaver";

export function isMascotId(value: unknown): value is MascotId {
  return typeof value === "string"
    && MASCOT_IDS.some((mascotId) => mascotId === value);
}
