import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { showToast } from "@/lib/toast-emitter";
import { invoke } from "@tauri-apps/api/core";
import { useLatestRequest } from "@/hooks/use-latest-request";
import {
  FORECAST_ANALYSIS_CREATED,
  FORECAST_ANALYSIS_DELETED,
  FORECAST_ANALYSIS_UPDATED,
  listenForecastAnalysisEvents,
} from "@/lib/forecast-analysis-events";
import { ForecastHistoryRow } from "./forecast-history-row";
import "../forecast-sections.css";
import "../forecast-history.css";
import { useForecastSessionId } from "../forecast-workspace-context";

export interface AnalysisMeta {
  id: string;
  name: string;
  created_at: string;
  model: string;
  horizon: number;
  points: number;
  mape: number | null;
  scenarios_count: number;
}

interface ForecastHistoryProps {
  onLoadAnalysis: (id: string) => void;
}

export function ForecastHistory({ onLoadAnalysis }: ForecastHistoryProps) {
  const sessionId = useForecastSessionId();
  const { t } = useTranslation();
  const [analyses, setAnalyses] = useState<AnalysisMeta[]>([]);
  const [unassigned, setUnassigned] = useState<AnalysisMeta[]>([]);
  const [search, setSearch] = useState("");
  /* Ne porte que l'échec du chargement : c'est l'état de la liste, et il doit
     rester visible sinon la zone est vide sans explication. Un renommage ou
     une suppression qui échoue est une action — elle passe par une
     notification qui s'efface. */
  const [error, setError] = useState<string | null>(null);
  const runLatest = useLatestRequest();

  const load = useCallback(async () => {
    try {
      if (!sessionId) throw new Error("missing_forecast_session");
      const next = await runLatest(
        () => Promise.all([
          invoke<AnalysisMeta[]>("list_forecast_analyses", { sessionId }),
          invoke<AnalysisMeta[]>("list_unassigned_forecast_analyses", { sessionId }),
        ]),
      );
      if (next === undefined) return;
      setAnalyses([...next[0]].sort((left, right) => (
        right.created_at.localeCompare(left.created_at)
      )));
      setUnassigned([...next[1]].sort((left, right) => (
        right.created_at.localeCompare(left.created_at)
      )));
      setError(null);
    } catch {
      setError(t("forecast.analysis.loadFailed"));
    }
  }, [runLatest, sessionId, t]);

  useEffect(() => {
    const timer = window.setTimeout(() => void load(), 0);
    const cleanup = listenForecastAnalysisEvents(
      [
        FORECAST_ANALYSIS_CREATED,
        FORECAST_ANALYSIS_UPDATED,
        FORECAST_ANALYSIS_DELETED,
      ],
      () => void load(),
    );
    return () => {
      window.clearTimeout(timer);
      cleanup();
    };
  }, [load]);

  const filtered = search
    ? analyses.filter((a) => a.name.toLowerCase().includes(search.toLowerCase()))
    : analyses;

  const handleRename = async (id: string, name: string) => {
    try {
      if (!sessionId) throw new Error("missing_forecast_session");
      const renamed = await invoke<AnalysisMeta>("rename_forecast_analysis", {
        sessionId,
        id,
        name,
      });
      setAnalyses((items) => items.map((item) => (item.id === id ? renamed : item)));
    } catch {
      showToast(t("forecast.history.renameFailed"), "error");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      if (!sessionId) throw new Error("missing_forecast_session");
      await invoke("delete_forecast_analysis", { sessionId, id });
      setAnalyses((items) => items.filter((item) => item.id !== id));
    } catch {
      showToast(t("forecast.history.deleteFailed"), "error");
    }
  };

  const handleClaim = async (id: string) => {
    try {
      if (!sessionId) throw new Error("missing_forecast_session");
      const claimed = await invoke<AnalysisMeta>("claim_legacy_forecast_analysis", {
        sessionId,
        id,
      });
      setUnassigned((items) => items.filter((item) => item.id !== id));
      setAnalyses((items) => [claimed, ...items]);
      onLoadAnalysis(id);
    } catch {
      showToast(t("forecast.history.claimFailed"), "error");
    }
  };

  return (
    <div className="fcs-root">
      <div className="fcs-toolbar">
        <span className="fcs-section-title">{t("forecast.nav.history")}</span>
      </div>
      <div className="fcs-content">
        <div className="fch-search">
          <input
            className="field fch-search-input"
            placeholder={t("forecast.history.searchPlaceholder")}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        {error && <p className="fch-error">{error}</p>}
        {unassigned.length > 0 && (
          <div className="fch-list">
            <p className="fcs-section-title">{t("forecast.history.unassigned")}</p>
            <p className="fcs-empty-sub">{t("forecast.history.unassignedHelp")}</p>
            {unassigned.map((analysis) => (
              <button
                key={analysis.id}
                className="btn btn-sm btn-secondary"
                type="button"
                onClick={() => void handleClaim(analysis.id)}
              >
                {t("forecast.history.claim", { name: analysis.name })}
              </button>
            ))}
          </div>
        )}
        {filtered.length === 0 ? (
          <div className="fcs-empty">
            <p className="fcs-empty-text">{t("forecast.history.empty")}</p>
          </div>
        ) : (
          <div className="fch-list">
            {filtered.map((a) => (
              <ForecastHistoryRow
                key={a.id}
                analysis={a}
                onLoad={onLoadAnalysis}
                onRename={handleRename}
                onDelete={handleDelete}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
