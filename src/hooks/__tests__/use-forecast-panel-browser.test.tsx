import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useForecastPanel } from "../use-forecast-panel";
import { DEFAULT_AGENT_LOCAL_WORKSPACE } from "@/types/navigation";

describe("useForecastPanel", () => {
  it("reflète directement l'état détenu par le workspace", () => {
    const { result } = renderHook(() => useForecastPanel({
      ...DEFAULT_AGENT_LOCAL_WORKSPACE,
      panelMode: "browser",
      forecastSection: "history",
      forecastNavOpen: true,
      forecastAnalysisId: "analysis-id",
    }));

    expect(result.current).toMatchObject({
      panelMode: "browser",
      activeSection: "history",
      navOpen: true,
      currentAnalysisId: "analysis-id",
    });
  });

  it("publie les changements forecast vers l'unique propriétaire", () => {
    const onWorkspaceChange = vi.fn();
    const { result } = renderHook(() => useForecastPanel(
      DEFAULT_AGENT_LOCAL_WORKSPACE,
      onWorkspaceChange,
    ));

    act(() => result.current.loadAnalysis("analysis-id"));
    expect(onWorkspaceChange).toHaveBeenCalledWith({
      forecastAnalysisId: "analysis-id",
      forecastSection: "view",
      panelMode: "forecast",
    });

    act(() => result.current.toggleNav());
    expect(onWorkspaceChange).toHaveBeenCalledWith({ forecastNavOpen: true });
  });
});
