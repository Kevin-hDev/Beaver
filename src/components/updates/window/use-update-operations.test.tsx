import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { UpdateOperationSnapshot } from "@/types/update-progress.generated";
import { useUpdateOperations } from "./use-update-operations";

const mocks = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));

const operation = (sequence: number): UpdateOperationSnapshot => ({
  id: "operation-1",
  sequence,
  kind: "app-release",
  label: "Beaver 1.4.2",
  status: "running",
  phase: "downloading",
  progressMode: "determinate",
  percent: sequence,
  queuePosition: null,
  canCancel: true,
  canRetry: false,
  isUpdate: null,
  errorKey: null,
});

describe("useUpdateOperations", () => {
  beforeEach(() => vi.clearAllMocks());

  it("écoute avant l'instantané et conserve la séquence la plus récente", async () => {
    let event: ((payload: { payload: UpdateOperationSnapshot }) => void) | undefined;
    let resolveSnapshot!: (value: UpdateOperationSnapshot[]) => void;
    const order: string[] = [];
    mocks.listen.mockImplementation((name: string, callback: (payload: { payload: UpdateOperationSnapshot }) => void) => {
      order.push("listen");
      if (name === "update-operation-changed") event = callback;
      return Promise.resolve(() => {});
    });
    mocks.invoke.mockImplementation((command: string) => {
      if (command !== "list_update_operations") return Promise.resolve();
      order.push("list");
      return new Promise((resolve) => { resolveSnapshot = resolve; });
    });
    const view = renderHook(() => useUpdateOperations());

    await waitFor(() => expect(order).toEqual(["listen", "listen", "list"]));
    act(() => event?.({ payload: operation(2) }));
    act(() => resolveSnapshot([operation(1)]));
    await waitFor(() => expect(view.result.current.operations[0]?.sequence).toBe(2));

    act(() => event?.({ payload: operation(1) }));
    expect(view.result.current.operations[0]?.sequence).toBe(2);
  });

  it("retire une opération terminale après son délai d'affichage", async () => {
    vi.useFakeTimers();
    try {
      mocks.listen.mockResolvedValue(() => {});
      mocks.invoke.mockImplementation((command: string) => Promise.resolve(
        command === "list_update_operations"
          ? [{ ...operation(2), status: "completed", canCancel: false }]
          : true,
      ));
      const view = renderHook(() => useUpdateOperations());
      await act(async () => { await Promise.resolve(); await Promise.resolve(); });
      expect(view.result.current.operations).toHaveLength(1);

      await act(async () => { await vi.advanceTimersByTimeAsync(4_000); });
      expect(mocks.invoke).toHaveBeenCalledWith("dismiss_update_operation", { id: "operation-1" });
      expect(view.result.current.operations).toHaveLength(0);
    } finally {
      vi.useRealTimers();
    }
  });

  it("conserve l'opération lorsque le backend refuse son retrait", async () => {
    mocks.listen.mockResolvedValue(() => {});
    mocks.invoke.mockImplementation((command: string) => Promise.resolve(
      command === "list_update_operations" ? [operation(2)] : false,
    ));
    const view = renderHook(() => useUpdateOperations());
    await waitFor(() => expect(view.result.current.operations).toHaveLength(1));

    await act(async () => { await view.result.current.dismiss("operation-1"); });

    expect(view.result.current.operations).toHaveLength(1);
  });

  it("conserve l'ancienne ligne jusqu'à la confirmation du backend", async () => {
    const failed = { ...operation(2), status: "failed" as const, canCancel: false, canRetry: true };
    let dismissed: ((event: { payload: string }) => void) | undefined;
    mocks.listen.mockImplementation((name: string, callback: (event: { payload: string }) => void) => {
      if (name === "update-operation-dismissed") dismissed = callback;
      return Promise.resolve(() => {});
    });
    mocks.invoke.mockImplementation((command: string) => Promise.resolve(
      command === "list_update_operations" ? [failed] : command !== "dismiss_update_operation",
    ));
    const view = renderHook(() => useUpdateOperations());
    await waitFor(() => expect(view.result.current.operations).toHaveLength(1));

    await act(async () => { await view.result.current.retry("operation-1"); });

    expect(view.result.current.operations).toHaveLength(1);
    act(() => dismissed?.({ payload: "operation-1" }));
    expect(view.result.current.operations).toHaveLength(0);
  });
});
