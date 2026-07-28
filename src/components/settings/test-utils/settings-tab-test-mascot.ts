let selectedMascotId = "cl-go-beaver";

export function mascotCommandResult(
  command: string,
  args?: Record<string, unknown>,
): { handled: boolean; value?: unknown } {
  if (command === "patch_mascot_settings") {
    const patch = args?.patch;
    const mascotId = patch && typeof patch === "object"
      ? (patch as Record<string, unknown>).mascot_id
      : null;
    if (
      mascotId === "cl-go-beaver"
      || mascotId === "circuit"
      || mascotId === "kova"
      || mascotId === "nival"
    ) {
      selectedMascotId = mascotId;
    }
  } else if (command !== "get_mascot_settings") {
    return { handled: false };
  }

  return {
    handled: true,
    value: {
      enabled: false,
      mascot_id: selectedMascotId,
      size_percent: 100,
      position: null,
    },
  };
}

export function resetMascotSettingsMock() {
  selectedMascotId = "cl-go-beaver";
}
