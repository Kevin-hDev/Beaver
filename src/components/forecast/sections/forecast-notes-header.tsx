import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import type { ForecastNotesAnalysis } from "./forecast-notes-types";

interface ForecastNotesHeaderProps {
  analysis: ForecastNotesAnalysis | null;
  noteCount: number;
  onCreate: () => void;
}

export function ForecastNotesHeader({
  analysis,
  noteCount,
  onCreate,
}: ForecastNotesHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="fcn-toolbar">
      <div className="fcn-title-wrap">
        <span className="fcs-section-title">{t("forecast.nav.notes")}</span>
        <span className="fcn-count">{t("forecast.notes.count", { count: noteCount })}</span>
        {analysis && <span className="fcn-analysis-name">{analysis.name}</span>}
        {analysis && (
          <span className="fcn-analysis-meta">
            {analysis.model} · H{analysis.horizon}
          </span>
        )}
      </div>
      <button className="fcn-new-btn" type="button" onClick={onCreate}>
        <Plus size="var(--icon-sm)" />
        <span>{t("forecast.notes.new")}</span>
      </button>
    </div>
  );
}
