import { act, renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EvaluationAnalysis } from "./forecast-evaluation-types";
import { useForecastEvaluation } from "./use-forecast-evaluation";

const ANALYSIS_ID = "550e8400-e29b-41d4-a716-446655440000";
const INITIAL: EvaluationAnalysis = { id: ANALYSIS_ID, model: "model-a", evaluation: null };
const WITH_ENSEMBLE: EvaluationAnalysis = {
  ...INITIAL,
  ensemble: {
    validation_status: "members_backtested_ensemble_not_backtested",
    members: [
      { model_id: "model-a", weight: 0.6, backtest_mase: 0.5 },
      { model_id: "model-b", weight: 0.4, backtest_mase: 0.8 },
    ],
  },
};

describe("useForecastEvaluation", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockResolvedValue(() => {});
  });

  it("creates a bounded automatic ensemble from saved backtests", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(INITIAL)
      .mockResolvedValueOnce(WITH_ENSEMBLE);
    const { result } = renderHook(() => useForecastEvaluation(ANALYSIS_ID));
    await waitFor(() => expect(result.current.analysis).toEqual(INITIAL));

    await act(async () => result.current.createEnsemble());

    expect(invoke).toHaveBeenLastCalledWith("create_forecast_ensemble", {
      analysisId: ANALYSIS_ID,
      modelIds: [],
    });
    expect(result.current.analysis?.ensemble?.members).toHaveLength(2);
    expect(result.current.ensembleFailed).toBe(false);
  });

  it("reports a generic failure without exposing backend details", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(INITIAL)
      .mockRejectedValueOnce(new Error("internal path"));
    const { result } = renderHook(() => useForecastEvaluation(ANALYSIS_ID));
    await waitFor(() => expect(result.current.analysis).toEqual(INITIAL));

    await act(async () => result.current.createEnsemble());

    expect(result.current.ensembleFailed).toBe(true);
  });
});
