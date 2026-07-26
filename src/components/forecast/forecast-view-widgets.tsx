import { buildPeriodMeta } from "./forecast-view-format";

export function PeriodCell({
  index,
  rawDate,
  endDate,
  frequency,
  locale,
}: {
  index: number;
  rawDate: string;
  endDate: string;
  frequency: string;
  locale: string;
}) {
  const period = buildPeriodMeta(index, rawDate, endDate, frequency, locale);

  return (
    <div className="fc-table-period">
      <span className="fc-table-step">{period.stepLabel}</span>
      <span className="fc-table-date">{period.secondaryLabel}</span>
    </div>
  );
}

export function ValueCell({
  unitLabel,
  formattedValue,
}: {
  unitLabel: string | null;
  formattedValue: string;
}) {
  const inlineUnit = unitLabel && unitLabel !== "€" ? unitLabel : null;

  return (
    <div className="fc-table-value-block">
      <span className="fc-table-value">{formattedValue}</span>
      {inlineUnit && <span className="fc-table-unit">{inlineUnit}</span>}
    </div>
  );
}

export function KpiRow({ metrics }: {
  metrics: { mape: number | null; mae: number | null; crps: number | null; bias: number | null };
}) {
  const kpis = [
    { label: "MAPE", value: metrics.mape, suffix: "%" },
    { label: "MAE", value: metrics.mae, suffix: "" },
    { label: "CRPS", value: metrics.crps, suffix: "" },
    { label: "Bias", value: metrics.bias, suffix: "" },
  ];

  return (
    <div className="fc-kpi-row">
      {kpis.map((kpi) => (
        <div key={kpi.label} className="fc-kpi-card">
          <span className="fc-kpi-label">{kpi.label}</span>
          <span className="fc-kpi-value">
            {kpi.value != null ? `${kpi.value.toFixed(2)}${kpi.suffix}` : "—"}
          </span>
        </div>
      ))}
    </div>
  );
}
