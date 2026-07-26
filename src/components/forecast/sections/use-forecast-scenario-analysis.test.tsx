import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useForecastScenarioAnalysis } from "./use-forecast-scenario-analysis";

const ANALYSIS_ID = "550e8400-e29b-41d4-a716-446655440000";

describe("useForecastScenarioAnalysis", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockReset();
  });

  it("reloads scenarios changed by the workbench or the LLM", async () => {
    let updated: ((event: { payload: { analysis_id: string } }) => void) | undefined;
    vi.mocked(listen).mockImplementation((_event, handler) => {
      updated = handler as typeof updated;
      return Promise.resolve(() => {});
    });
    vi.mocked(invoke)
      .mockResolvedValueOnce({ scenarios: [] })
      .mockResolvedValueOnce({ scenarios: [{ id: "scenario-1" }] });
    const onLoaded = vi.fn();
    const onFailed = vi.fn();

    renderHook(() => useForecastScenarioAnalysis({
      analysisId: ANALYSIS_ID,
      onLoaded,
      onFailed,
    }));
    await waitFor(() => expect(onLoaded).toHaveBeenCalledTimes(1));

    act(() => updated?.({ payload: { analysis_id: ANALYSIS_ID } }));

    await waitFor(() => expect(onLoaded).toHaveBeenCalledTimes(2));
    expect(onFailed).not.toHaveBeenCalled();
  });
});
