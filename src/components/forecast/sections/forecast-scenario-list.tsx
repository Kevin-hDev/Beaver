import { useTranslation } from "react-i18next";
import { ForecastScenarioRow } from "./forecast-scenario-row";
import type { ForecastScenario } from "./forecast-scenario-types";

interface ForecastScenarioListProps {
  scenarios: ForecastScenario[];
  activeScenarioId: string | null;
  onSelect: (scenario: ForecastScenario) => void;
  onEdit: (scenario: ForecastScenario) => void;
  onDelete: (scenarioId: string) => void;
}

export function ForecastScenarioList({
  scenarios,
  activeScenarioId,
  onSelect,
  onEdit,
  onDelete,
}: ForecastScenarioListProps) {
  const { t } = useTranslation();
  if (!scenarios.length) {
    return (
      <div className="fcs-empty">
        <p className="fcs-empty-text">{t("forecast.scenarios.empty")}</p>
        <p className="fcs-empty-sub">{t("forecast.scenarios.emptySub")}</p>
      </div>
    );
  }

  return (
    <div className="fcs-list">
      {scenarios.map((scenario) => (
        <ForecastScenarioRow
          key={scenario.id}
          scenario={scenario}
          active={scenario.id === activeScenarioId}
          onSelect={onSelect}
          onEdit={onEdit}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
