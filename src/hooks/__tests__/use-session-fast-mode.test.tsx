import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import { useSessionFastMode } from "../use-session-fast-mode";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  return { promise, resolve, reject };
}

describe("useSessionFastMode", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("persiste puis recharge avant de libérer la session", async () => {
    const mutation = deferred<boolean>();
    vi.mocked(invoke).mockReturnValue(mutation.promise);
    const refresh = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useSessionFastMode(refresh));

    let request!: Promise<void>;
    act(() => {
      request = result.current.setFastMode("session-1", true);
    });
    expect(result.current.isFastModePending("session-1")).toBe(true);
    expect(invoke).toHaveBeenCalledWith("set_session_fast_mode", {
      id: "session-1",
      enabled: true,
    });

    mutation.resolve(true);
    await act(async () => request);

    expect(refresh).toHaveBeenCalledTimes(1);
    expect(result.current.isFastModePending("session-1")).toBe(false);
  });

  it("ignore une seconde mutation simultanée de la même session", async () => {
    const mutation = deferred<boolean>();
    vi.mocked(invoke).mockReturnValue(mutation.promise);
    const { result } = renderHook(() => useSessionFastMode(vi.fn()));

    let first!: Promise<void>;
    let duplicate!: Promise<void>;
    act(() => {
      first = result.current.setFastMode("session-1", true);
      duplicate = result.current.setFastMode("session-1", false);
    });

    expect(invoke).toHaveBeenCalledTimes(1);
    await duplicate;
    mutation.resolve(true);
    await act(async () => first);
  });

  it("borne les mutations à 32 sans inventer une erreur de sauvegarde", async () => {
    const requests = Array.from({ length: 32 }, () => deferred<boolean>());
    vi.mocked(invoke).mockImplementation(() => requests[vi.mocked(invoke).mock.calls.length - 1].promise);
    const refresh = vi.fn().mockResolvedValue(undefined);
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const { result } = renderHook(() => useSessionFastMode(refresh));
    let pending: Promise<void>[] = [];

    act(() => {
      pending = requests.map((_, index) =>
        result.current.setFastMode(`session-${index}`, true),
      );
    });
    await act(async () => {
      await result.current.setFastMode("session-32", true);
    });

    expect(invoke).toHaveBeenCalledTimes(32);
    expect(result.current.isFastModePending("session-32")).toBe(false);
    expect(showToast).not.toHaveBeenCalled();
    expect(consoleError).not.toHaveBeenCalled();

    requests.forEach((request) => request.resolve(true));
    await act(async () => Promise.all(pending));
    consoleError.mockRestore();
  });

  it("affiche une erreur générique puis recharge après un vrai échec IPC", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("internal path must stay hidden"));
    const refresh = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useSessionFastMode(refresh));

    await act(async () => {
      await result.current.setFastMode("session-1", true);
    });

    expect(showToast).toHaveBeenCalledWith("errors.sessionSaveFailed", "error");
    expect(refresh).toHaveBeenCalledTimes(1);
    await waitFor(() => {
      expect(result.current.isFastModePending("session-1")).toBe(false);
    });
  });
});
