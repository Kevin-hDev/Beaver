import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import { act, renderHook } from "@testing-library/react";
import {
  installExtensionUiStartupCapture,
  parseExtensionUiStartupState,
} from "./extension-ui-startup";
import { useExtensionUiStartup } from "@/hooks/use-extension-ui-startup";

const normal = {
  mode: { kind: "normal" },
  bootstrapResolved: true,
  thirdPartyLoadingAllowed: true,
  showRecoveryDialog: false,
  showSafeBanner: false,
  canRetry: false,
};

describe("extension UI startup boundary", () => {
  it("keeps the frontend fallback closed when Rust rejects the safe transition", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("internal sentinel"));
    const fallback = {
      ...normal,
      mode: { kind: "safe", reason: "invalidMarker" } as const,
      thirdPartyLoadingAllowed: false,
      showRecoveryDialog: true,
      showSafeBanner: true,
    };
    const view = renderHook(() => useExtensionUiStartup(fallback));

    await act(async () => {
      expect(await view.result.current.continueSafe()).toBe(false);
    });

    expect(view.result.current.state).toEqual(fallback);
    expect(view.result.current.error).toBe(true);
  });
  it("rejects unknown and malformed native projections", () => {
    expect(parseExtensionUiStartupState(normal)).toEqual(normal);
    expect(() => parseExtensionUiStartupState({ ...normal, extra: true })).toThrow();
    expect(() => parseExtensionUiStartupState({
      ...normal,
      mode: { kind: "pendingInterruptedUi", extensionId: "../escape", stage: "mount", attempts: 1 },
    })).toThrow();
    expect(() => parseExtensionUiStartupState({
      ...normal,
      mode: { kind: "pendingInterruptedUi", extensionId: "com.example.ui", stage: "register", attempts: 1 },
    })).toThrow();
  });

  it("captures Shift before resolution and removes the listener", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "get_extension_ui_startup_state") {
        return Promise.resolve({
          ...normal,
          mode: { kind: "awaitingWayland" },
          bootstrapResolved: false,
          thirdPartyLoadingAllowed: false,
        });
      }
      if (command === "confirm_extension_ui_wayland_shift") {
        return Promise.resolve({
          ...normal,
          mode: { kind: "safe", reason: "shift" },
          thirdPartyLoadingAllowed: false,
          showSafeBanner: true,
        });
      }
      return Promise.reject(new Error("unexpected command"));
    });
    const pending = installExtensionUiStartupCapture(document, window);
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Shift" }));
    const state = await pending;

    expect(state.mode).toEqual({ kind: "safe", reason: "shift" });
    expect(invoke).toHaveBeenCalledWith("confirm_extension_ui_wayland_shift", {
      shiftPressed: true,
    });
    const calls = vi.mocked(invoke).mock.calls.length;
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Shift" }));
    expect(invoke).toHaveBeenCalledTimes(calls);
  });
});
