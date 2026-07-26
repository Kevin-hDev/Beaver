import type { TFunction } from "i18next";
import type { VariableInsight } from "./forecast-analysis-types";

interface ForecastAnalysisVariableListProps {
  variables: VariableInsight[];
  t: TFunction;
}

export function ForecastAnalysisVariableList({ variables, t }: ForecastAnalysisVariableListProps) {
  if (!variables.length) {
    return <p className="fca-empty-line">{t("forecast.analysis.noVariables")}</p>;
  }
  const max = Math.max(...variables.map((variable) => variable.score), 1);
  return (
    <div className="fca-variable-list">
      {variables.map((variable) => (
        <div key={variable.name} className="fca-variable">
          <div className="fca-variable-head">
            <span>{variable.name}</span>
            <span>{variable.detail}</span>
          </div>
          <div className="fca-variable-track">
            <span style={{ width: `${Math.max(0, (variable.score / max) * 100)}%` }} />
          </div>
        </div>
      ))}
    </div>
  );
}
