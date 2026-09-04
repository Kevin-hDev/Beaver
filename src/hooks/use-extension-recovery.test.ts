import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { EMPTY_EXTENSION_RECOVERY, useExtensionRecovery } from "./use-extension-recovery";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("useExtensionRecovery", () => {
  beforeEach(() => vi.clearAllMocks());

  it("conserve l'incident et rétablit les actions après une erreur", async () => {
    const incident = {
      ...EMPTY_EXTENSION_RECOVERY,
      extensionId: "com.example.crash",
      stage: "activate" as const,
      attempts: 1,
      canRetry: true,
    };
    vi.mocked(invoke).mockResolvedValue(incident);
    const run = vi.fn()
      .mockRejectedValueOnce(new Error("internal path"))
      .mockResolvedValueOnce(undefined);
    const runHost = async (operation: () => Promise<void>) => operation();
    const setOperationError = vi.fn();
    const view = renderHook(() => useExtensionRecovery(
      run,
      runHost,
      setOperationError,
    ));

    await act(() => view.result.current.refreshRecovery());
    await expect(act(() => view.result.current.keepDisabled(incident.extensionId)))
      .rejects.toThrow("internal path");
    expect(view.result.current.recovery).toEqual(incident);
    expect(view.result.current.recoveryBusy).toBe(false);

    await act(() => view.result.current.keepDisabled(incident.extensionId));
    expect(run).toHaveBeenCalledTimes(2);
  });
});
