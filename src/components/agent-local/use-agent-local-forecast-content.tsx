import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ForecastSection } from "@/hooks/use-forecast-panel";
import { ForecastPanel } from "@/components/forecast/forecast-panel";
import { openForecastDocsWindow } from "@/components/forecast/open-forecast-docs";
import { openForecastWorkbench } from "@/components/forecast/workbench/open-forecast-workbench";
import { syncOpenForecastWorkbenchContext } from "@/components/forecast/workbench/forecast-workbench-context-sync";
import i18n from "@/i18n";
import { showToast } from "@/lib/toast-emitter";

interface ForecastNavigation {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  setSection: (section: ForecastSection) => void;
  toggleNav: () => void;
  loadAnalysis: (id: string) => void;
  closeAnalysis: () => void;
}

interface FilePreviewControls {
  fullscreen: boolean;
  setFullscreen: (value: boolean) => void;
  setExtraWidth: (width: number) => void;
}

interface Args {
  forecastNav: ForecastNavigation;
  filePreview: FilePreviewControls;
  sessionId: string | null;
}

export function useAgentLocalForecastContent({
  forecastNav,
  filePreview,
  sessionId,
}: Args) {
  const [fullscreenSwitching, setFullscreenSwitching] = useState(false);
  const fullscreenTimerRef = useRef<number | null>(null);
  const setPreviewExtraWidth = filePreview.setExtraWidth;

  useEffect(() => {
    setPreviewExtraWidth(0);
    return () => setPreviewExtraWidth(0);
  }, [setPreviewExtraWidth]);

  useEffect(() => {
    if (!sessionId) return;
    void syncOpenForecastWorkbenchContext({
      sessionId,
      analysisId: forecastNav.currentAnalysisId,
      title: i18n.t("forecast.workbench.windowTitle"),
    }).catch(() => showToast(i18n.t("forecast.workbench.syncFailed")));
  }, [forecastNav.currentAnalysisId, sessionId]);

  const handlePreviewFullscreenChange = useCallback((value: boolean) => {
    if (fullscreenTimerRef.current !== null) window.clearTimeout(fullscreenTimerRef.current);
    setFullscreenSwitching(true);
    filePreview.setFullscreen(value);
    fullscreenTimerRef.current = window.setTimeout(() => setFullscreenSwitching(false), 80);
  }, [filePreview]);

  useEffect(() => () => {
    if (fullscreenTimerRef.current !== null) window.clearTimeout(fullscreenTimerRef.current);
  }, []);

  const handleOpenForecastDocs = useCallback(() => {
    void openForecastDocsWindow(i18n.t("forecast.docs.windowTitle"))
      .catch(() => showToast(i18n.t("forecast.docs.openFailed")));
  }, []);
  const handleOpenForecastWorkbench = useCallback(() => {
    if (!sessionId) {
      showToast(i18n.t("forecast.workbench.openFailed"));
      return;
    }
    void openForecastWorkbench({
      sessionId,
      analysisId: forecastNav.currentAnalysisId,
      title: i18n.t("forecast.workbench.windowTitle"),
    }).catch(() => showToast(i18n.t("forecast.workbench.openFailed")));
  }, [forecastNav.currentAnalysisId, sessionId]);

  const forecastContent = useMemo(() => (
    <ForecastPanel
      activeSection={forecastNav.activeSection}
      navOpen={forecastNav.navOpen}
      currentAnalysisId={forecastNav.currentAnalysisId}
      onSectionChange={forecastNav.setSection}
      onToggleNav={forecastNav.toggleNav}
      onLoadAnalysis={forecastNav.loadAnalysis}
      onCloseAnalysis={forecastNav.closeAnalysis}
      onOpenWorkbench={handleOpenForecastWorkbench}
    />
  ), [forecastNav, handleOpenForecastWorkbench]);

  return {
    forecastContent, fullscreenSwitching, handleOpenForecastDocs, handlePreviewFullscreenChange,
  };
}
