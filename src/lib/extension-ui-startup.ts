import { invoke } from "@tauri-apps/api/core";
import { UI_LOADING_STAGES } from "@/types/extension-ui-contract.generated";
import { LIMITS, type ExtensionUiStartupMode, type ExtensionUiStartupState } from "@/types/extensions";
import { isExtensionIdentifier } from "@/lib/extension-records";

const CAPTURE_TIMEOUT_MS = 1_500;
const ROOT_KEYS = [
  "mode", "bootstrapResolved", "thirdPartyLoadingAllowed",
  "showRecoveryDialog", "showSafeBanner", "canRetry",
] as const;
const SAFE_REASONS = ["argument", "shift", "invalidMarker", "recoveryChoice"] as const;

export const NORMAL_EXTENSION_UI_STARTUP: ExtensionUiStartupState = {
  mode: { kind: "normal" },
  bootstrapResolved: true,
  thirdPartyLoadingAllowed: true,
  showRecoveryDialog: false,
  showSafeBanner: false,
  canRetry: false,
};

export const FAIL_CLOSED_EXTENSION_UI_STARTUP: ExtensionUiStartupState = {
  mode: { kind: "safe", reason: "invalidMarker" },
  bootstrapResolved: true,
  thirdPartyLoadingAllowed: false,
  showRecoveryDialog: true,
  showSafeBanner: true,
  canRetry: false,
};

export function parseExtensionUiStartupState(value: unknown): ExtensionUiStartupState {
  const input = objectWithKeys(value, ROOT_KEYS);
  for (const key of ROOT_KEYS.slice(1)) {
    if (typeof input[key] !== "boolean") throwInvalid();
  }
  return {
    mode: parseMode(input.mode),
    bootstrapResolved: input.bootstrapResolved as boolean,
    thirdPartyLoadingAllowed: input.thirdPartyLoadingAllowed as boolean,
    showRecoveryDialog: input.showRecoveryDialog as boolean,
    showSafeBanner: input.showSafeBanner as boolean,
    canRetry: input.canRetry as boolean,
  };
}

function parseMode(value: unknown): ExtensionUiStartupMode {
  if (!value || typeof value !== "object" || Array.isArray(value)) throwInvalid();
  const kind = (value as Record<string, unknown>).kind;
  if (kind === "normal" || kind === "awaitingWayland") {
    objectWithKeys(value, ["kind"]);
    return { kind };
  }
  if (kind === "safe") {
    const input = objectWithKeys(value, ["kind", "reason"]);
    if (!SAFE_REASONS.includes(input.reason as typeof SAFE_REASONS[number])) throwInvalid();
    return { kind, reason: input.reason as typeof SAFE_REASONS[number] };
  }
  if (kind === "pendingInterruptedUi") {
    const input = objectWithKeys(value, ["kind", "extensionId", "stage", "attempts"]);
    validateIdentityAndAttempt(input);
    if (!UI_LOADING_STAGES.includes(input.stage as typeof UI_LOADING_STAGES[number])) throwInvalid();
    return input as ExtensionUiStartupMode;
  }
  if (kind === "retryInterruptedUi") {
    const input = objectWithKeys(value, ["kind", "extensionId", "attempts"]);
    validateIdentityAndAttempt(input);
    return input as ExtensionUiStartupMode;
  }
  throwInvalid();
}

function validateIdentityAndAttempt(input: Record<string, unknown>): void {
  if (typeof input.extensionId !== "string"
    || input.extensionId.length > LIMITS.maxIdentifierChars
    || !isExtensionIdentifier(input.extensionId)
    || !Number.isInteger(input.attempts)
    || (input.attempts as number) < 1
    || (input.attempts as number) > 3) throwInvalid();
}

function objectWithKeys(value: unknown, keys: readonly string[]): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throwInvalid();
  const input = value as Record<string, unknown>;
  const actual = Object.keys(input).sort();
  const expected = [...keys].sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    throwInvalid();
  }
  return input;
}

export async function installExtensionUiStartupCapture(
  documentTarget: Document = document,
  windowTarget: Window = window,
): Promise<ExtensionUiStartupState> {
  let shiftPressed = false;
  let resolveShift: ((pressed: boolean) => void) | undefined;
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key !== "Shift") return;
    shiftPressed = true;
    resolveShift?.(true);
  };
  documentTarget.addEventListener("keydown", onKeyDown, true);
  try {
    const initial = parseExtensionUiStartupState(
      await invoke<unknown>("get_extension_ui_startup_state"),
    );
    if (initial.bootstrapResolved) return initial;
    const pressed = shiftPressed || await new Promise<boolean>((resolve) => {
      const timeout = windowTarget.setTimeout(() => finish(false), CAPTURE_TIMEOUT_MS);
      const finish = (pressed: boolean) => {
        windowTarget.clearTimeout(timeout);
        resolve(pressed);
      };
      resolveShift = finish;
    });
    return parseExtensionUiStartupState(await invoke<unknown>(
      "confirm_extension_ui_wayland_shift",
      { shiftPressed: pressed },
    ));
  } finally {
    resolveShift = undefined;
    documentTarget.removeEventListener("keydown", onKeyDown, true);
  }
}

function throwInvalid(): never {
  throw new Error("invalid_extension_ui_startup_response");
}
