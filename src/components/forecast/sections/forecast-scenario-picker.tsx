import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChevronDown } from "@/components/ui/icons";
import { useTranslation } from "react-i18next";
import "./forecast-scenario-picker.css";

interface AnalysisMeta {
  id: string;
  name: string;
  created_at: string;
  model: string;
  horizon: number;
  points: number;
  scenarios_count: number;
}

interface ForecastScenarioPickerProps {
  open: boolean;
  currentAnalysisId: string;
  onSelectAnalysis: (id: string) => void;
}

export function ForecastScenarioPicker({
  open,
  currentAnalysisId,
  onSelectAnalysis,
}: ForecastScenarioPickerProps) {
  const { t, i18n } = useTranslation();
  const [analyses, setAnalyses] = useState<AnalysisMeta[]>([]);
  const [openMonths, setOpenMonths] = useState<string[]>([]);

  useEffect(() => {
    let active = true;
    void invoke<AnalysisMeta[]>("list_forecast_analyses")
      .then((list) => {
        if (!active) return;
        const sorted = [...list].sort((a, b) => b.created_at.localeCompare(a.created_at));
        setAnalyses(sorted);
        const currentMonth = sorted.find((item) => item.id === currentAnalysisId)?.created_at.slice(0, 7);
        if (currentMonth) setOpenMonths([currentMonth]);
      })
      .catch(() => {
        if (active) setAnalyses([]);
      });
    return () => {
      active = false;
    };
  }, [currentAnalysisId]);

  const groups = useMemo(() => groupByMonth(analyses), [analyses]);

  return (
    <aside className={`fsp-panel ${open ? "open" : ""}`} aria-hidden={!open}>
      <div className="fsp-header">
        <span className="fsp-title">{t("forecast.scenarios.predictions")}</span>
      </div>
      <div className="fsp-body">
        {groups.length === 0 ? (
          <div className="fsp-empty">{t("forecast.scenarios.noPredictions")}</div>
        ) : (
          groups.map((group) => {
            const expanded = openMonths.includes(group.id);
            return (
              <div key={group.id} className="fsp-group">
                <button
                  className="fsp-group-btn"
                  type="button"
                  onClick={() =>
                    setOpenMonths((current) =>
                      current.includes(group.id)
                        ? current.filter((id) => id !== group.id)
                        : [...current, group.id]
                    )
                  }
                >
                  <span>{formatMonth(group.id, i18n.language)}</span>
                  <ChevronDown size="var(--icon-sm)" className={`fsp-chevron ${expanded ? "is-open" : ""}`} />
                </button>
                <div className={`fsp-group-items ${expanded ? "is-open" : ""}`}>
                  <div className="fsp-group-inner">
                    {group.items.map((analysis) => (
                      <button
                        key={analysis.id}
                        className={`fsp-item ${analysis.id === currentAnalysisId ? "is-active" : ""}`}
                        type="button"
                        onClick={() => onSelectAnalysis(analysis.id)}
                      >
                        <div className="fsp-item-main">
                          <span className="fsp-item-name">{analysis.name}</span>
                          <span className="fsp-item-meta">
                            {analysis.model} · {analysis.points} · H{analysis.horizon}
                          </span>
                        </div>
                        <div className="fsp-item-side">
                          {analysis.scenarios_count > 0 && <span className="fsp-scenario-dot" />}
                          <span className="fsp-item-date">
                            {formatDay(analysis.created_at, i18n.language)}
                          </span>
                        </div>
                      </button>
                    ))}
                  </div>
                </div>
              </div>
            );
          })
        )}
      </div>
    </aside>
  );
}

function groupByMonth(analyses: AnalysisMeta[]) {
  const groups = new Map<string, AnalysisMeta[]>();
  for (const analysis of analyses) {
    const month = analysis.created_at.slice(0, 7);
    const items = groups.get(month) ?? [];
    items.push(analysis);
    groups.set(month, items);
  }
  return Array.from(groups.entries()).map(([id, items]) => ({ id, items }));
}

function formatMonth(value: string, locale: string): string {
  const date = new Date(`${value}-01T00:00:00Z`);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(locale, { month: "long", year: "numeric" }).format(date);
}

function formatDay(value: string, locale: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(locale, {
    day: "2-digit",
    month: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(date);
}
