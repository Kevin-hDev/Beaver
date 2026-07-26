import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { useAgentLocalForecastContent } from "../use-agent-local-forecast-content";

interface WorkbenchSyncInput {
  sessionId: string;
  analysisId: string | null;
  title: string;
}

const mocks = vi.hoisted(() => ({
  syncWorkbench: vi.fn<(input: WorkbenchSyncInput) => Promise<void>>(),
}));

vi.mock("@/components/forecast/workbench/forecast-workbench-context-sync", () => ({
  syncOpenForecastWorkbenchContext: mocks.syncWorkbench,
}));

describe("useAgentLocalForecastContent", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.syncWorkbench.mockResolvedValue(undefined);
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("applies fullscreen changes and clears the transition state", () => {
    const setFullscreen = vi.fn();
    const setExtraWidth = vi.fn();
    const { result } = renderHook(() => useAgentLocalForecastContent({
      forecastNav: {
        activeSection: "view",
        navOpen: true,
        currentAnalysisId: null,
        setSection: vi.fn(),
        toggleNav: vi.fn(),
        loadAnalysis: vi.fn(),
        closeAnalysis: vi.fn(),
      },
      filePreview: {
        fullscreen: false,
        setFullscreen,
        setExtraWidth,
      },
      sessionId: "550e8400-e29b-41d4-a716-446655440000",
    }));

    expect(setExtraWidth).toHaveBeenCalledWith(0);
    act(() => result.current.handlePreviewFullscreenChange(true));
    expect(setFullscreen).toHaveBeenCalledWith(true);
    expect(result.current.fullscreenSwitching).toBe(true);

    act(() => {
      vi.advanceTimersByTime(80);
    });
    expect(result.current.fullscreenSwitching).toBe(false);
  });

  it("synchronizes a new panel analysis with the open workbench", () => {
    const baseNav = {
      activeSection: "view" as const,
      navOpen: false,
      setSection: vi.fn(),
      toggleNav: vi.fn(),
      loadAnalysis: vi.fn(),
      closeAnalysis: vi.fn(),
    };
    const filePreview = {
      fullscreen: false,
      setFullscreen: vi.fn(),
      setExtraWidth: vi.fn(),
    };
    const { rerender } = renderHook(
      ({ analysisId }: { analysisId: string | null }) =>
        useAgentLocalForecastContent({
          forecastNav: { ...baseNav, currentAnalysisId: analysisId },
          filePreview,
          sessionId: "550e8400-e29b-41d4-a716-446655440000",
        }),
      { initialProps: { analysisId: null as string | null } },
    );

    rerender({ analysisId: "123e4567-e89b-12d3-a456-426614174000" });

    expect(mocks.syncWorkbench).toHaveBeenLastCalledWith({
      sessionId: "550e8400-e29b-41d4-a716-446655440000",
      analysisId: "123e4567-e89b-12d3-a456-426614174000",
      title: i18n.t("forecast.workbench.windowTitle"),
    });
  });
});
