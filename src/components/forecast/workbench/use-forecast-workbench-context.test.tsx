import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ForecastWorkbenchSnapshot } from "./forecast-workbench-types";
import { useForecastWorkbenchContext } from "./use-forecast-workbench-context";

const FIRST: ForecastWorkbenchSnapshot = {
  context: {
    session_id: "550e8400-e29b-41d4-a716-446655440000",
    analysis_id: null,
    revision: 1,
  },
  draft: { section: "data", revision: 1 },
  analysis_name: null,
};

describe("useForecastWorkbenchContext", () => {
  let contextChanged: (() => void) | null;
  let eventSnapshot: ForecastWorkbenchSnapshot;

  beforeEach(() => {
    contextChanged = null;
    eventSnapshot = FIRST;
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockImplementation((_event, handler) => {
      contextChanged = () => handler({ payload: eventSnapshot } as never);
      return Promise.resolve(() => {});
    });
  });

  it("applies a complete snapshot event without another backend fetch", async () => {
    const second: ForecastWorkbenchSnapshot = {
      ...FIRST,
      context: { ...FIRST.context, revision: 2 },
      analysis_name: "Prévision juillet",
    };
    vi.mocked(invoke).mockResolvedValueOnce(FIRST);
    const { result } = renderHook(() => useForecastWorkbenchContext());

    await waitFor(() => expect(result.current.snapshot).toEqual(FIRST));
    eventSnapshot = second;
    act(() => contextChanged?.());
    await waitFor(() => expect(result.current.snapshot).toEqual(second));

    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("fails closed when the snapshot is malformed", async () => {
    vi.mocked(invoke).mockResolvedValue({ context: { session_id: "../bad" } });
    const { result } = renderHook(() => useForecastWorkbenchContext());

    await waitFor(() => expect(result.current.failed).toBe(true));
    expect(result.current.snapshot).toBeNull();
  });

  it("does not let initial hydration overwrite a newer event", async () => {
    const second: ForecastWorkbenchSnapshot = {
      ...FIRST,
      context: { ...FIRST.context, revision: 2 },
      analysis_name: "Prévision récente",
    };
    let resolveInitial!: (value: ForecastWorkbenchSnapshot) => void;
    vi.mocked(invoke).mockReturnValue(new Promise((resolve) => {
      resolveInitial = resolve;
    }));
    const { result } = renderHook(() => useForecastWorkbenchContext());
    eventSnapshot = second;

    act(() => contextChanged?.());
    await waitFor(() => expect(result.current.snapshot).toEqual(second));
    act(() => resolveInitial(FIRST));

    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.snapshot).toEqual(second);
  });
});
