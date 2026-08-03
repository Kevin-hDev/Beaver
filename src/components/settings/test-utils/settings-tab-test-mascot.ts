import { isMascotId, type MascotId } from "@/types/mascot";

let selectedMascotId: MascotId = "cl-go-beaver";
let sizePercent = 100;

export function mascotCommandResult(
  command: string,
  args?: Record<string, unknown>,
): { handled: boolean; value?: unknown } {
  if (command === "patch_mascot_settings") {
    const patch = args?.patch;
    const mascotId = patch && typeof patch === "object"
      ? (patch as Record<string, unknown>).mascot_id
      : null;
    if (isMascotId(mascotId)) {
      selectedMascotId = mascotId;
    }
    const nextSize = patch && typeof patch === "object"
      ? (patch as Record<string, unknown>).size_percent
      : null;
    if (typeof nextSize === "number") {
      sizePercent = nextSize;
    }
  } else if (command !== "get_mascot_settings") {
    return { handled: false };
  }

  return {
    handled: true,
    value: {
      enabled: false,
      mascot_id: selectedMascotId,
      size_percent: sizePercent,
      position: null,
    },
  };
}

export function resetMascotSettingsMock() {
  selectedMascotId = "cl-go-beaver";
  sizePercent = 100;
}
