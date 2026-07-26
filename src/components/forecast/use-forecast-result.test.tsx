import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useForecastResult } from "./use-forecast-result";

const ANALYSIS_ID = "550e8400-e29b-41d4-a716-446655440000";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe("useForecastResult", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockReset();
  });

  it("keeps the newest refresh when responses finish out of order", async () => {
    const first = deferred<{ value: string }>();
    const second = deferred<{ value: string }>();
    let updated: ((event: { payload: { analysis_id: string } }) => void) | undefined;
    vi.mocked(invoke)
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    vi.mocked(listen).mockImplementation((_name, handler) => {
      updated = handler as typeof updated;
      return Promise.resolve(() => {});
    });
    const { result } = renderHook(
      () => useForecastResult<{ value: string }>(ANALYSIS_ID, "failed"),
    );
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
    act(() => updated?.({ payload: { analysis_id: ANALYSIS_ID } }));
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));

    await act(async () => {
      second.resolve({ value: "new" });
      await second.promise;
    });
    await waitFor(() => expect(result.current.data?.value).toBe("new"));
    await act(async () => {
      first.resolve({ value: "old" });
      await first.promise;
    });

    expect(result.current.data?.value).toBe("new");
  });

  it("ne montre pas l'ancienne analyse pendant le chargement suivant", async () => {
    const next = deferred<{ value: string }>();
    vi.mocked(invoke)
      .mockResolvedValueOnce({ value: "first" })
      .mockReturnValueOnce(next.promise);
    vi.mocked(listen).mockResolvedValue(() => {});
    const { result, rerender } = renderHook(
      ({ id }) => useForecastResult<{ value: string }>(id, "failed"),
      { initialProps: { id: "analysis-a" } },
    );
    await waitFor(() => expect(result.current.data?.value).toBe("first"));

    rerender({ id: "analysis-b" });

    expect(result.current.data).toBeNull();
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(2));
  });
});
