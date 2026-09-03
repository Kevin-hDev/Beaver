/* @vitest-environment jsdom */
import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useCatalogSync } from "../use-catalog-sync";

const empty = (revision: number) => ({ revision, contributions: [] });

describe("useCatalogSync", () => {
  let changed: ((event: { payload: number }) => void) | undefined;
  const unlisten = vi.fn();

  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockReset();
    changed = undefined;
    unlisten.mockReset();
    vi.mocked(listen).mockImplementation((_event, handler) => {
      changed = handler as (event: { payload: number }) => void;
      return Promise.resolve(unlisten);
    });
  });

  it("keeps the last healthy catalog when a refresh is invalid", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(empty(1))
      .mockResolvedValueOnce({ revision: 2, contributions: "invalid" });
    const { result, unmount } = renderHook(() => useCatalogSync());

    await waitFor(() => expect(result.current.kind).toBe("empty"));
    act(() => changed?.({ payload: 2 }));
    await waitFor(() => expect(result.current.kind).toBe("stale-error"));
    expect(result.current.snapshot?.revision).toBe(1);

    unmount();
    await waitFor(() => expect(unlisten).toHaveBeenCalledOnce());
  });

  it("coalesces concurrent notifications without losing the newest revision", async () => {
    let release!: (value: unknown) => void;
    vi.mocked(invoke)
      .mockImplementationOnce(() => new Promise((resolve) => { release = resolve; }))
      .mockResolvedValueOnce(empty(4));
    const { result } = renderHook(() => useCatalogSync());
    await waitFor(() => expect(changed).toBeTypeOf("function"));

    act(() => {
      changed?.({ payload: 2 });
      changed?.({ payload: 3 });
      changed?.({ payload: 4 });
      release(empty(1));
    });

    await waitFor(() => expect(result.current.snapshot?.revision).toBe(4));
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("ignores an older response", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(empty(3))
      .mockResolvedValueOnce(empty(2));
    const { result } = renderHook(() => useCatalogSync());
    await waitFor(() => expect(result.current.snapshot?.revision).toBe(3));

    act(() => changed?.({ payload: 4 }));
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
    expect(result.current.snapshot?.revision).toBe(3);
  });

  it("preserves the authoritative snapshot identity for an equal revision", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(empty(3))
      .mockResolvedValueOnce(empty(3));
    const { result } = renderHook(() => useCatalogSync());
    await waitFor(() => expect(result.current.snapshot?.revision).toBe(3));
    const first = result.current.snapshot;

    act(() => changed?.({ payload: 3 }));
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));

    expect(result.current.snapshot).toBe(first);
  });

  it("retries when the backend has not reached the requested revision", async () => {
    vi.useFakeTimers();
    vi.mocked(invoke)
      .mockResolvedValueOnce(empty(1))
      .mockResolvedValueOnce(empty(1))
      .mockResolvedValueOnce(empty(2));
    const { result } = renderHook(() => useCatalogSync());
    await vi.waitFor(() => expect(result.current.snapshot?.revision).toBe(1));

    act(() => changed?.({ payload: 2 }));
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
    await act(() => vi.runOnlyPendingTimersAsync());
    await vi.waitFor(() => expect(result.current.snapshot?.revision).toBe(2));
    vi.useRealTimers();
  });
});
