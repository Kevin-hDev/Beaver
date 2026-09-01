import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import { useForecastSessionId } from "./forecast-workspace-context";

interface ForecastExportResult {
  kind: "file" | "clipboard";
  format: string;
  file_path?: string | null;
  content?: string | null;
}

export function useForecastExport() {
  const { t } = useTranslation();
  const sessionId = useForecastSessionId();

  return useCallback(async (format: string, analysisId: string) => {
    try {
      if (!sessionId) throw new Error("missing_forecast_session");
      const result = await invoke<ForecastExportResult>("export_forecast_analysis", {
        sessionId,
        analysisId,
        format,
      });
      if (result.kind === "clipboard") {
        await navigator.clipboard.writeText(result.content ?? "");
        showToast(t("forecast.export.copied"), "success");
        return;
      }
      showToast(t("forecast.export.saved"), "success");
    } catch {
      showToast(t("forecast.export.failed"), "error");
    }
  }, [sessionId, t]);
}
