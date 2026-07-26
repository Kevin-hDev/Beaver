import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useForecastPanel } from "../use-forecast-panel";

describe("useForecastPanel browser mode", () => {
  it("restaure le navigateur séparément pour chaque conversation", () => {
    localStorage.setItem("fc-panel-session-browser", JSON.stringify({
      activeSection: "view",
      navOpen: false,
      currentAnalysisId: null,
      panelMode: "browser",
    }));

    const { result } = renderHook(() => useForecastPanel("session-browser"));

    expect(result.current.panelMode).toBe("browser");
  });

  it("remplace l'ancien onglet Analyse par la vue principale", () => {
    localStorage.setItem("fc-panel-session-analysis", JSON.stringify({
      activeSection: "analysis",
      navOpen: false,
      currentAnalysisId: "analysis-id",
      panelMode: "forecast",
    }));

    const { result } = renderHook(() => useForecastPanel("session-analysis"));

    expect(result.current.activeSection).toBe("view");
    expect(result.current.currentAnalysisId).toBe("analysis-id");
  });

  it.each(["scenarios", "notes"])(
    "remplace l'ancien onglet %s par la vue principale",
    (activeSection) => {
      localStorage.setItem("fc-panel-session-moved", JSON.stringify({
        activeSection,
        navOpen: false,
        currentAnalysisId: "analysis-id",
        panelMode: "forecast",
      }));

      const { result } = renderHook(() => useForecastPanel("session-moved"));

      expect(result.current.activeSection).toBe("view");
    },
  );
});
