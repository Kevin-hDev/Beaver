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
      mode: { kind: "pendingInterruptedUi", extensionId: "../escape", stage: "mount", attempts: 1, startedAt: "2026-09-03T10:00:00Z" },
    })).toThrow();
    expect(() => parseExtensionUiStartupState({
      ...normal,
      mode: { kind: "pendingInterruptedUi", extensionId: "com.example.ui", stage: "register", attempts: 1, startedAt: "2026-09-03T10:00:00Z" },
    })).toThrow();
    expect(() => parseExtensionUiStartupState({
      ...normal,
      mode: {
        kind: "pendingInterruptedUi",
        extensionId: "com.example.ui",
        stage: "mount",
        attempts: 1,
        startedAt: "not-a-date",
      },
    })).toThrow();
    expect(() => parseExtensionUiStartupState({
      ...normal,
      mode: {
        kind: "pendingInterruptedUi",
        extensionId: "com.example.ui",
        stage: "mount",
        attempts: 1,
        startedAt: "2026-09-03",
      },
    })).toThrow();
  });

  it("preserves the bounded incident until it is explicitly discarded", async () => {
    const pending = parseExtensionUiStartupState({
      ...normal,
      mode: {
        kind: "pendingInterruptedUi",
        extensionId: "com.example.ui",
        stage: "mount",
        attempts: 1,
        startedAt: "2026-09-03T10:00:00Z",
      },
      thirdPartyLoadingAllowed: false,
      showRecoveryDialog: true,
    });
    vi.mocked(invoke).mockResolvedValueOnce({
      ...normal,
      mode: { kind: "safe", reason: "recoveryChoice" },
      thirdPartyLoadingAllowed: false,
      showSafeBanner: true,
    });
    const view = renderHook(() => useExtensionUiStartup(pending));

    expect(view.result.current.incident?.extensionId).toBe("com.example.ui");
    await act(async () => {
      expect(await view.result.current.discardInterrupted("com.example.ui"))
        .toBe(true);
    });
    expect(invoke).toHaveBeenCalledWith("discard_interrupted_extension_ui_marker", {
      extensionId: "com.example.ui",
    });
    expect(view.result.current.incident).toBeNull();
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
