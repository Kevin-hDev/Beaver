export function updateErrorKey(error: unknown): string {
  switch (error) {
    case "update-download-error":
      return "errors.downloadFailed";
    case "update-write-error":
      return "errors.updatePrepareFailed";
    case "update-install-error":
      return "errors.updateInstallFailed";
    default:
      return "errors.updateFailed";
  }
}
