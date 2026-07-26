import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { openForecastWorkbench } from "./open-forecast-workbench";

const mocks = vi.hoisted(() => ({
  setTitle: vi.fn(),
  show: vi.fn(),
  setFocus: vi.fn(),
  getByLabel: vi.fn(),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: class {
    static getByLabel = mocks.getByLabel;
  },
}));

const SESSION_ID = "550e8400-e29b-41d4-a716-446655440000";
const ANALYSIS_ID = "123e4567-e89b-12d3-a456-426614174000";

describe("openForecastWorkbench", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.getByLabel.mockResolvedValue({
      setTitle: mocks.setTitle,
      show: mocks.show,
      setFocus: mocks.setFocus,
    });
    vi.mocked(invoke).mockResolvedValue({
      context: { session_id: SESSION_ID, analysis_id: ANALYSIS_ID, revision: 2 },
      draft: { section: "data", revision: 1 },
      analysis_name: "Prévision juillet",
    });
  });

  it("reuses the single window after updating its validated context", async () => {
    await openForecastWorkbench({
      sessionId: SESSION_ID,
      analysisId: ANALYSIS_ID,
      title: "Espace Forecast",
    });

    expect(invoke).toHaveBeenCalledWith("set_forecast_workbench_context", {
      sessionId: SESSION_ID,
      analysisId: ANALYSIS_ID,
    });
    expect(mocks.setTitle).toHaveBeenCalledWith("Espace Forecast — Prévision juillet");
    expect(mocks.show).toHaveBeenCalledOnce();
    expect(mocks.setFocus).toHaveBeenCalledOnce();
  });

  it("does not open a window for an invalid backend snapshot", async () => {
    vi.mocked(invoke).mockResolvedValue({ context: { session_id: "../bad" } });

    await expect(openForecastWorkbench({
      sessionId: SESSION_ID,
      analysisId: null,
      title: "Espace Forecast",
    })).rejects.toThrow("forecast-workbench-unavailable");

    expect(mocks.getByLabel).not.toHaveBeenCalled();
  });
});
