import { act, renderHook } from "@testing-library/react";
import { listen } from "@tauri-apps/api/event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { useFileTree } from "@/hooks/use-file-tree";
import { useForecastPanel } from "@/hooks/use-forecast-panel";
import { DEFAULT_APP_NAV } from "@/types/navigation";
import { useAgentLocalControlledPanels } from "../use-agent-local-controlled-panels";

function dependencies() {
  return {
    filePreview: {
      setOpen: vi.fn(),
      setFullscreen: vi.fn(),
    } as unknown as ReturnType<typeof useFilePreview>,
    fileTree: {
      setOpen: vi.fn(),
      closeTree: vi.fn(),
    } as unknown as ReturnType<typeof useFileTree>,
    forecast: {
      setPanelMode: vi.fn(),
      setSection: vi.fn(),
      loadAnalysis: vi.fn(),
      focusAnalysis: vi.fn(),
      closeAnalysis: vi.fn(),
    } as unknown as ReturnType<typeof useForecastPanel>,
  };
}

describe("useAgentLocalControlledPanels", () => {
  it("sort du plein écran quand Forecast devient actif", () => {
    const deps = dependencies();
    const onNavChange = vi.fn();
    const { result } = renderHook(() => useAgentLocalControlledPanels({
      ...deps,
      sessionId: null,
      navState: { ...DEFAULT_APP_NAV.agentLocal, previewFullscreen: true },
      onNavChange,
    }));

    act(() => result.current.forecastNav.setPanelMode("forecast"));

    expect(deps.filePreview.setFullscreen).toHaveBeenCalledWith(false);
    expect(onNavChange).toHaveBeenCalledWith({
      panelMode: "forecast",
      previewFullscreen: false,
    });
  });

  it.each(["preview", "browser"] as const)(
    "charge une analyse depuis %s sans masquer la conversation",
    (panelMode) => {
      const deps = dependencies();
      const onNavChange = vi.fn();
      const { result } = renderHook(() => useAgentLocalControlledPanels({
        ...deps,
        sessionId: null,
        navState: { ...DEFAULT_APP_NAV.agentLocal, panelMode, previewFullscreen: true },
        onNavChange,
      }));

      act(() => result.current.forecastNav.loadAnalysis("analysis-id"));

      expect(deps.forecast.loadAnalysis).toHaveBeenCalledWith("analysis-id");
      expect(onNavChange).toHaveBeenCalledWith({
        previewOpen: true,
        forecastAnalysisId: "analysis-id",
        forecastSection: "view",
        panelMode: "forecast",
        previewFullscreen: false,
      });
    },
  );

  it("affiche automatiquement une analyse créée par le LLM", () => {
    let analysisCreated: ((event: { payload: unknown }) => void) | null = null;
    vi.mocked(listen).mockImplementationOnce((_event, callback) => {
      analysisCreated = callback as (event: { payload: unknown }) => void;
      return Promise.resolve(() => {});
    });
    const deps = dependencies();
    const onNavChange = vi.fn();
    const { result } = renderHook(() => {
      const [navState, setNavState] = useState(DEFAULT_APP_NAV.agentLocal);
      const forecast = useForecastPanel("session-id");
      return useAgentLocalControlledPanels({
        ...deps,
        forecast,
        sessionId: "session-id",
        navState,
        onNavChange: (patch) => {
          onNavChange(patch);
          setNavState((current) => ({ ...current, ...patch }));
        },
      });
    });

    act(() => analysisCreated?.({
      payload: { analysis_id: "llm-analysis-id", session_id: "session-id" },
    }));

    expect(result.current.forecastNav.currentAnalysisId).toBe("llm-analysis-id");
    expect(deps.filePreview.setOpen).toHaveBeenCalledWith(true);
    expect(onNavChange).toHaveBeenCalledWith({
      previewOpen: true,
      forecastAnalysisId: "llm-analysis-id",
      forecastSection: "view",
      panelMode: "forecast",
      previewFullscreen: false,
    });
  });

  it("ignore les événements Forecast étrangers ou invalides", () => {
    let analysisCreated: ((event: { payload: unknown }) => void) | null = null;
    vi.mocked(listen).mockImplementationOnce((_event, callback) => {
      analysisCreated = callback as (event: { payload: unknown }) => void;
      return Promise.resolve(() => {});
    });
    const deps = dependencies();
    const onNavChange = vi.fn();
    renderHook(() => useAgentLocalControlledPanels({
      ...deps,
      sessionId: "session-id",
      navState: DEFAULT_APP_NAV.agentLocal,
      onNavChange,
    }));

    act(() => analysisCreated?.({
      payload: { analysis_id: "other-analysis", session_id: "other-session" },
    }));
    act(() => analysisCreated?.({
      payload: { analysis_id: "a".repeat(129), session_id: "session-id" },
    }));

    expect(deps.forecast.loadAnalysis).not.toHaveBeenCalled();
    expect(deps.filePreview.setOpen).not.toHaveBeenCalled();
    expect(onNavChange).not.toHaveBeenCalled();
  });
});
