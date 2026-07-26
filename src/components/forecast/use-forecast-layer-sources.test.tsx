import { useState } from "react";
import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ForecastLayerState } from "./forecast-layer-matrix";
import { useForecastLayerSources } from "./use-forecast-layer-sources";

const ANALYSIS_ID = "550e8400-e29b-41d4-a716-446655440000";

describe("useForecastLayerSources", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockReset();
  });

  it("refreshes comparison layers after an ensemble is created in another window", async () => {
    let updated: ((event: { payload: { analysis_id: string } }) => void) | undefined;
    vi.mocked(listen).mockImplementation((_event, handler) => {
      updated = handler as typeof updated;
      return Promise.resolve(() => {});
    });
    vi.mocked(invoke)
      .mockResolvedValueOnce({ scenarios: [], covariates_used: [], ensemble: null })
      .mockResolvedValueOnce({ scenarios: [], covariates_used: [], ensemble: {} });

    const { result } = renderHook(() => {
      const [layers, setLayers] = useState<ForecastLayerState>({});
      return { layers, hook: useForecastLayerSources(ANALYSIS_ID, setLayers) };
    });
    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));

    act(() => updated?.({ payload: { analysis_id: ANALYSIS_ID } }));

    await waitFor(() => expect(result.current.hook.sources.comparisonLayers).toHaveLength(1));
    expect(result.current.layers["scenario-ensemble"]).toBe(true);
  });
});
