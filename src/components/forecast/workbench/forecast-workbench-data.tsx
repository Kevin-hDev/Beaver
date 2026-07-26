import { useTranslation } from "react-i18next";
import { useForecastResult } from "../use-forecast-result";
import { ForecastWorkbenchDataTable } from "./forecast-workbench-data-table";
import { buildDataPreview, type WorkbenchInputData } from "./forecast-workbench-data-utils";
import "./forecast-workbench-data.css";

interface DataIssue {
  code: string;
  severity: "error" | "warning";
  count: number;
}

interface DataProfile {
  valid: boolean;
  row_count: number;
  history_points: number;
  future_rows: number;
  series_count: number;
  missing_periods: number;
  outlier_count: number;
  date_column: string;
  covariate_columns: string[];
  issues: DataIssue[];
}

interface DataAnalysis {
  target_column: string;
  frequency: string;
  input_summary: { points: number };
  input_data: WorkbenchInputData & { covariate_columns?: string[] };
  data_profile?: DataProfile | null;
}

export function ForecastWorkbenchData({ analysisId }: { analysisId: string }) {
  const { t } = useTranslation();
  const { data, error } = useForecastResult<DataAnalysis>(
    analysisId,
    t("forecast.workbench.data.loadFailed"),
  );
  if (error) return <div className="fcw-inline-error">{error}</div>;
  if (!data) return <div className="fcwd-loading"><div className="fc-skeleton" /></div>;

  const profile = data.data_profile;
  const preview = buildDataPreview(data.input_data, data.target_column);
  const covariates = profile?.covariate_columns ?? data.input_data.covariate_columns ?? [];
  const stats = [
    ["rows", profile?.row_count ?? preview.totalRows],
    ["historyPoints", profile?.history_points ?? data.input_summary.points],
    ["futureRows", profile?.future_rows ?? 0],
    ["series", profile?.series_count ?? 1],
    ["missingPeriods", profile?.missing_periods ?? 0],
    ["outliers", profile?.outlier_count ?? 0],
  ] as const;

  return (
    <div className="fcwd-root">
      <div className="fcwd-stats">
        {stats.map(([key, value]) => (
          <div className="fcwd-stat" key={key}>
            <span>{t(`forecast.workbench.data.${key}`)}</span>
            <strong>{value}</strong>
          </div>
        ))}
      </div>
      <section className="fcwd-mapping">
        <div><span>{t("forecast.workbench.data.target")}</span><strong>{data.target_column}</strong></div>
        <div><span>{t("forecast.workbench.data.date")}</span><strong>{profile?.date_column ?? data.input_data.date_column ?? "—"}</strong></div>
        <div><span>{t("forecast.workbench.data.frequency")}</span><strong>{data.frequency}</strong></div>
        <div>
          <span>{t("forecast.workbench.data.covariates")}</span>
          <strong>{covariates.length ? covariates.join(", ") : t("forecast.workbench.data.none")}</strong>
        </div>
      </section>
      <section className={`fcwd-quality ${profile
        ? profile.valid ? "is-valid" : "is-warning"
        : ""}`}>
        <div>
          <span>{t("forecast.workbench.data.quality")}</span>
          <strong>{!profile
            ? t("forecast.workbench.data.notAvailable")
            : profile.valid
              ? t("forecast.workbench.data.valid")
              : t("forecast.workbench.data.needsAttention")}</strong>
        </div>
        {profile?.issues.length ? (
          <ul>
            {profile.issues.map((issue) => (
              <li key={`${issue.code}-${issue.severity}`}>
                {t(issue.severity === "error"
                  ? "forecast.workbench.data.issueError"
                  : "forecast.workbench.data.issueWarning")} · {issue.count}
              </li>
            ))}
          </ul>
        ) : null}
      </section>
      <ForecastWorkbenchDataTable {...preview} />
    </div>
  );
}
