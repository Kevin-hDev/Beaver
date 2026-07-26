import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Tooltip } from "@/components/ui/tooltip";
import {
  newestForecastAnalyses,
  type ForecastAnalysisMeta,
} from "./forecast-empty-recent";
import "./forecast-empty.css";

interface ForecastEmptyProps {
  onLoadAnalysis: (id: string) => void;
  onImportFile?: (path: string) => void;
  error?: string | null;
}

export function ForecastEmpty({ onLoadAnalysis, onImportFile, error }: ForecastEmptyProps) {
  const { t } = useTranslation();
  const [recent, setRecent] = useState<ForecastAnalysisMeta[]>([]);

  useEffect(() => {
    invoke<ForecastAnalysisMeta[]>("list_forecast_analyses")
      .then(setRecent)
      .catch(() => setRecent([]));
  }, []);

  const handleImport = useCallback(async () => {
    const path = await open({
      filters: [{ name: "Data", extensions: ["csv", "tsv", "xlsx", "xls", "ods", "xlsm"] }],
      multiple: false,
    });
    if (path && onImportFile) onImportFile(path);
  }, [onImportFile]);

  return (
    <div className="fc-empty">
      <div className="fc-empty-icon">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none" stroke="var(--ink-faint)"
          strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M6 36L14 24L22 30L32 16L42 22" />
          <path d="M32 16L42 16L42 22" opacity="0.5" />
          <line x1="6" y1="42" x2="42" y2="42" opacity="0.3" />
        </svg>
      </div>
      <p className="fc-empty-title">{t("forecast.noAnalysis")}</p>
      <p className="fc-empty-sub">{t("forecast.askAgent")}</p>
      {error && <p className="fc-empty-error">{error}</p>}
      <div className="fc-empty-actions">
        <button className="fc-empty-btn fc-empty-btn-primary" onClick={() => void handleImport()}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor"
            strokeWidth="1.5" strokeLinecap="round">
            <circle cx="7" cy="7" r="5.5" />
            <path d="M7 4.5v5M4.5 7h5" />
          </svg>
          {t("forecast.importFile")}
        </button>
        <Tooltip label={t("forecast.comingSoon")}>
          <button className="fc-empty-btn" disabled>
            {t("forecast.pasteData")}
          </button>
        </Tooltip>
        <Tooltip label={t("forecast.comingSoon")}>
          <button className="fc-empty-btn" disabled>
            {t("forecast.fromUrl")}
          </button>
        </Tooltip>
      </div>
      {recent.length > 0 && (
        <div className="fc-recent">
          <p className="fc-recent-title">{t("forecast.recentAnalyses")}</p>
          {newestForecastAnalyses(recent).map((a) => (
            <button key={a.id} className="fc-recent-item" onClick={() => onLoadAnalysis(a.id)}>
              <span className="fc-recent-name">{a.name}</span>
              <span className="fc-recent-meta">
                {a.model} · H{a.horizon}
                {a.mape != null && ` · ${a.mape.toFixed(1)}%`}
              </span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
