import { useTranslation } from "react-i18next";
import { ForecastEvaluationView } from "../evaluation/forecast-evaluation-view";
import { ForecastWorkbenchData } from "./forecast-workbench-data";
import { ForecastWorkbenchForecast } from "./forecast-workbench-forecast";
import { ForecastWorkbenchNotes } from "./forecast-workbench-notes";
import { ForecastWorkbenchReport } from "./forecast-workbench-report";
import { ForecastWorkbenchScenarios } from "./forecast-workbench-scenarios";
import type {
  ForecastWorkbenchSection,
  ForecastWorkbenchSnapshot,
} from "./forecast-workbench-types";

interface ForecastWorkbenchSectionProps {
  section: ForecastWorkbenchSection;
  snapshot: ForecastWorkbenchSnapshot;
}

export function ForecastWorkbenchSectionContent({
  section,
  snapshot,
}: ForecastWorkbenchSectionProps) {
  const { t } = useTranslation();
  const analysisId = snapshot.context.analysis_id;
  if (!analysisId) {
    return (
      <div className="fcw-foundation">
        <span>{t("forecast.workbench.evaluation.noAnalysis")}</span>
        <p>{t("forecast.workbench.evaluation.noAnalysisDescription")}</p>
      </div>
    );
  }
  if (section === "data") {
    return <ForecastWorkbenchData key={analysisId} analysisId={analysisId} />;
  }
  if (section === "forecast") {
    return <ForecastWorkbenchForecast key={analysisId} analysisId={analysisId} />;
  }
  if (section === "scenarios") {
    return <ForecastWorkbenchScenarios key={analysisId} analysisId={analysisId} />;
  }
  if (section === "notes") {
    return <ForecastWorkbenchNotes key={analysisId} analysisId={analysisId} />;
  }
  if (section === "report") {
    return <ForecastWorkbenchReport key={analysisId} analysisId={analysisId} />;
  }
  if (section === "evaluation" || section === "comparison") {
    return (
      <ForecastEvaluationView
        key={analysisId}
        analysisId={analysisId}
        mode={section}
      />
    );
  }
  return null;
}
