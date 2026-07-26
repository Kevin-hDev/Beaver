import { useTranslation } from "react-i18next";
import type { ForecastWorkbenchSection } from "./forecast-workbench-types";

const SECTIONS: ForecastWorkbenchSection[] = [
  "data",
  "forecast",
  "evaluation",
  "comparison",
  "scenarios",
  "notes",
  "report",
];

interface ForecastWorkbenchNavProps {
  active: ForecastWorkbenchSection;
  onChange: (section: ForecastWorkbenchSection) => void;
}

export function ForecastWorkbenchNav({ active, onChange }: ForecastWorkbenchNavProps) {
  const { t } = useTranslation();
  return (
    <nav className="fcw-nav" aria-label={t("forecast.workbench.navigation")}>
      {SECTIONS.map((section) => (
        <button
          key={section}
          type="button"
          className={`fcw-nav-item ${active === section ? "is-active" : ""}`}
          aria-current={active === section ? "page" : undefined}
          onClick={() => onChange(section)}
        >
          {t(`forecast.workbench.sections.${section}`)}
        </button>
      ))}
    </nav>
  );
}
