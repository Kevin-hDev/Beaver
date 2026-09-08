import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useBrowserFavicons } from "../use-browser-favicons";
import type { BrowserTabState } from "../browser-types";

const id = "1".repeat(32);
const png = "iVBORw0KGgo=";
const tabs: BrowserTabState[] = [{ id, title: "", url: "https://example.org/", released: false,
  loading: false, canGoBack: false, canGoForward: false }];
const snapshot = (revision: number, conversationId = "a", icons = [{ tabId: id, pngBase64: png }]) =>
  ({ eventVersion: 1, revision, conversationId, icons });
const deferred = <T,>() => {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
};

describe("useBrowserFavicons", () => {
  let deliver: (event: { payload: unknown }) => void;
  const unlisten = vi.fn();
  beforeEach(() => {
    unlisten.mockClear();
    vi.mocked(listen).mockImplementation((_event, callback) => {
      deliver = callback as typeof deliver;
      return Promise.resolve(unlisten);
    });
    vi.mocked(invoke).mockResolvedValue(snapshot(1));
  });
  it("subscribes before reading and ignores an older initial response", async () => {
    const pending = deferred<unknown>();
    vi.mocked(invoke).mockImplementation(() => pending.promise);
    const { result } = renderHook(() => useBrowserFavicons("a", true, tabs));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("browser_favicon_snapshot", { conversationId: "a" }));
    act(() => deliver({ payload: snapshot(3) }));
    await act(async () => { pending.resolve(snapshot(2, "a", [])); await Promise.resolve(); });
    expect(result.current.get(id)).toBe(png);
    act(() => deliver({ payload: snapshot(4, "a", []) }));
    expect(result.current.size).toBe(0);
    act(() => deliver({ payload: snapshot(3) }));
    expect(result.current.size).toBe(0);
  });
  it("ignores old conversation callbacks and filters closed/released tabs", async () => {
    const { result, rerender } = renderHook(({ conversation, current }) =>
      useBrowserFavicons(conversation, true, current), { initialProps: { conversation: "a", current: tabs } });
    await waitFor(() => expect(result.current.get(id)).toBe(png));
    const old = deliver;
    rerender({ conversation: "b", current: tabs });
    act(() => old({ payload: snapshot(999) }));
    expect(result.current.size).toBe(0);
    act(() => deliver({ payload: snapshot(5, "b") }));
    expect(result.current.get(id)).toBe(png);
    rerender({ conversation: "b", current: [{ ...tabs[0], released: true }] });
    expect(result.current.size).toBe(0);
  });
  it("disposes an asynchronously installed listener without reading after unmount", async () => {
    const pending = deferred<() => void>();
    vi.mocked(listen).mockReturnValue(pending.promise);
    vi.mocked(invoke).mockClear();
    const { unmount } = renderHook(() => useBrowserFavicons("a", true, tabs));
    unmount();
    await act(async () => { pending.resolve(unlisten); await Promise.resolve(); });
    expect(unlisten).toHaveBeenCalledTimes(1);
    expect(invoke).not.toHaveBeenCalled();
  });
  it("keeps a newer event if the initial read fails", async () => {
    const pending = deferred<unknown>();
    vi.mocked(invoke).mockReturnValue(pending.promise);
    const { result } = renderHook(() => useBrowserFavicons("a", true, tabs));
    await act(async () => { await Promise.resolve(); });
    act(() => deliver({ payload: snapshot(3) }));
    await act(async () => { pending.reject(new Error("read failed")); await Promise.resolve(); });
    expect(result.current.get(id)).toBe(png);
  });
  it("falls back to empty state when subscription fails", async () => {
    vi.mocked(listen).mockRejectedValue(new Error("listener unavailable"));
    const { result } = renderHook(() => useBrowserFavicons("a", true, tabs));
    await act(async () => { await Promise.resolve(); });
    expect(result.current.size).toBe(0);
  });
  it("reports failures once without including rejected data", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const pending = deferred<unknown>();
    vi.mocked(invoke).mockReturnValue(pending.promise);
    const { unmount } = renderHook(() => useBrowserFavicons("a", true, tabs));
    await act(async () => { await Promise.resolve(); });
    act(() => deliver({ payload: snapshot(2, "b") }));
    expect(warn).not.toHaveBeenCalled();
    act(() => deliver({ payload: { conversationId: "a", secret: "private" } }));
    act(() => deliver({ payload: null }));
    await act(async () => { pending.reject(new Error("private")); await Promise.resolve(); });
    expect(warn).toHaveBeenCalledExactlyOnceWith("[browser] favicon synchronization unavailable");
    unmount();
    warn.mockRestore();
  });
  it("ignores duplicate revisions and late events after disabling", async () => {
    const { result, rerender } = renderHook(({ active }) => useBrowserFavicons("a", active, tabs),
      { initialProps: { active: true } });
    await waitFor(() => expect(result.current.size).toBe(1));
    act(() => deliver({ payload: snapshot(1, "a", []) }));
    expect(result.current.size).toBe(1);
    rerender({ active: false });
    act(() => deliver({ payload: snapshot(9) }));
    expect(result.current.size).toBe(0);
    expect(unlisten).toHaveBeenCalledTimes(1);
  });
  it("restores from a snapshot on remount and stays empty when disabled", async () => {
    const first = renderHook(() => useBrowserFavicons("a", true, tabs));
    await waitFor(() => expect(first.result.current.size).toBe(1));
    first.unmount();
    const second = renderHook(({ active }) => useBrowserFavicons("a", active, tabs), { initialProps: { active: false } });
    expect(second.result.current.size).toBe(0);
    second.rerender({ active: true });
    await waitFor(() => expect(second.result.current.size).toBe(1));
  });
});
