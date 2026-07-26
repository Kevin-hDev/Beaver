import type { TFunction } from "i18next";
import { PeriodCell } from "../forecast-view-widgets";
import type { ForecastComparisonRow } from "./forecast-comparison-types";
import "./forecast-comparison-table.css";

interface ForecastComparisonTableProps {
  rows: ForecastComparisonRow[];
  endDate: string;
  frequency: string;
  locale: string;
  t: TFunction;
  formatValue: (value: number) => string;
}

export function ForecastComparisonTable({
  rows,
  endDate,
  frequency,
  locale,
  t,
  formatValue,
}: ForecastComparisonTableProps) {
  return (
    <div className="fccmp-table">
      <div className="fccmp-table-head">
        <span>{t("forecast.view.period")}</span>
        <div className="fccmp-table-metrics">
          <span>{t("forecast.comparisons.base")}</span>
          <span className="fccmp-table-sep">·</span>
          <span>{t("forecast.comparisons.diff")}</span>
          <span className="fccmp-table-sep">-</span>
          <span>{t("forecast.comparisons.delta")}</span>
        </div>
      </div>
      <div className="fccmp-table-body">
        {rows.map((row) => (
          <div key={`${row.date}-${row.index}`} className="fccmp-table-row">
            <PeriodCell
              index={row.index}
              rawDate={row.date}
              endDate={endDate}
              frequency={frequency}
              locale={locale}
            />
            <div className="fccmp-table-metrics">
              <span>{formatValue(row.baseValue)}</span>
              <span className="fccmp-table-sep">·</span>
              <span>{formatValue(row.compareValue)}</span>
              <span className="fccmp-table-sep">-</span>
              <span className={row.delta >= 0 ? "is-up" : "is-down"}>
                {formatValue(row.delta)} · {row.deltaPercent.toFixed(1)}%
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
