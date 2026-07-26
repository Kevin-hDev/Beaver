import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { syncOpenForecastWorkbenchContext } from "./forecast-workbench-context-sync";

const mocks = vi.hoisted(() => ({
  getByLabel: vi.fn(),
  setTitle: vi.fn(),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: class {
    static getByLabel = mocks.getByLabel;
  },
}));

const SESSION_ID = "550e8400-e29b-41d4-a716-446655440000";
const ANALYSIS_ID = "123e4567-e89b-12d3-a456-426614174000";
const SECOND_ANALYSIS_ID = "223e4567-e89b-12d3-a456-426614174000";

describe("syncOpenForecastWorkbenchContext", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue({
      context: { session_id: SESSION_ID, analysis_id: ANALYSIS_ID, revision: 2 },
      draft: { section: "forecast", revision: 1 },
      analysis_name: "Prévision juillet",
    });
  });

  it("does nothing while the workbench is closed", async () => {
    mocks.getByLabel.mockResolvedValue(null);

    await syncOpenForecastWorkbenchContext({
      sessionId: SESSION_ID,
      analysisId: ANALYSIS_ID,
      title: "Espace Forecast",
    });

    expect(invoke).not.toHaveBeenCalled();
  });

  it("updates the open workbench and its title", async () => {
    mocks.getByLabel.mockResolvedValue({ setTitle: mocks.setTitle });

    await syncOpenForecastWorkbenchContext({
      sessionId: SESSION_ID,
      analysisId: ANALYSIS_ID,
      title: "Espace Forecast",
    });

    expect(invoke).toHaveBeenCalledWith("set_forecast_workbench_context", {
      sessionId: SESSION_ID,
      analysisId: ANALYSIS_ID,
    });
    expect(mocks.setTitle).toHaveBeenCalledWith(
      "Espace Forecast — Prévision juillet",
    );
  });

  it("finishes on the latest analysis when selections change quickly", async () => {
    let releaseLookup = () => {};
    mocks.getByLabel
      .mockImplementationOnce(() => new Promise((resolve) => {
        releaseLookup = () => resolve({ setTitle: mocks.setTitle });
      }))
      .mockResolvedValue({ setTitle: mocks.setTitle });

    const first = syncOpenForecastWorkbenchContext({
      sessionId: SESSION_ID,
      analysisId: ANALYSIS_ID,
      title: "Espace Forecast",
    });
    const second = syncOpenForecastWorkbenchContext({
      sessionId: SESSION_ID,
      analysisId: SECOND_ANALYSIS_ID,
      title: "Espace Forecast",
    });
    releaseLookup();
    await Promise.all([first, second]);

    expect(invoke).toHaveBeenLastCalledWith("set_forecast_workbench_context", {
      sessionId: SESSION_ID,
      analysisId: SECOND_ANALYSIS_ID,
    });
  });
});
