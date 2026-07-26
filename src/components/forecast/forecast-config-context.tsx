import { useTranslation } from "react-i18next";
import type { ForecastModelEntry } from "./forecast-model-meta";
import type { ForecastContextProfile } from "./forecast-context-profile";

interface ForecastConfigContextProps {
  profile: ForecastContextProfile;
  horizon: number;
  model: ForecastModelEntry | null;
}

export function ForecastConfigContext({
  profile,
  horizon,
  model,
}: ForecastConfigContextProps) {
  const { t } = useTranslation();
  const status = buildContextStatus(profile, horizon, model);

  return (
    <div className="fcc-field">
      <span className="fcc-label">{t("forecast.config.dataPhases")}</span>
      <div className="fcc-context-grid">
        <Stat
          label={t("forecast.config.historyRows")}
          value={profile.historyRows.toString()}
        />
        <Stat
          label={t("forecast.config.futureRows")}
          value={profile.futureRows.toString()}
        />
        <Stat
          label={t("forecast.config.seriesCount")}
          value={profile.seriesCount.toString()}
        />
        <Stat
          label={t("forecast.config.selectedContext")}
          value={profile.selectedCovariates.toString()}
        />
        <Stat
          label={t("forecast.config.detectedFutureContext")}
          value={profile.futureContextColumns.length.toString()}
        />
      </div>

      {profile.futureContextColumns.length > 0 && (
        <div className="fcc-context-columns">
          <span className="fcc-context-columns-label">
            {t("forecast.config.futureContextColumns")}
          </span>
          <div className="fcc-chips">
            {profile.futureContextColumns.map((column) => (
              <span key={column} className="fcc-chip fcc-chip-static">
                {column}
              </span>
            ))}
          </div>
        </div>
      )}

      {status && (
        <div className={`fcc-callout fcc-callout-${status.tone}`}>
          <span className="fcc-callout-title">{t(status.titleKey)}</span>
          <span className="fcc-callout-body">
            {t(status.bodyKey, status.values)}
          </span>
        </div>
      )}
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="fcc-context-stat">
      <span className="fcc-context-stat-label">{label}</span>
      <span className="fcc-context-stat-value">{value}</span>
    </div>
  );
}

function buildContextStatus(
  profile: ForecastContextProfile,
  horizon: number,
  model: ForecastModelEntry | null,
):
  | {
      tone: "info" | "warning" | "ok";
      titleKey: string;
      bodyKey: string;
      values?: Record<string, unknown>;
    }
  | null {
  const supportsPast = Boolean(model?.capabilities?.past_covariates);
  const supportsFuture = Boolean(model?.capabilities?.future_covariates);

  if (
    profile.futureRows > 0 &&
    profile.futureRowsPerSeries != null &&
    profile.futureRowsPerSeries !== horizon
  ) {
    return {
      tone: "warning",
      titleKey: "forecast.config.messages.futureRowsMismatchTitle",
      bodyKey: "forecast.config.messages.futureRowsMismatchBody",
      values: { future: profile.futureRowsPerSeries, horizon },
    };
  }
  if (profile.futureRows > 0 && profile.futureRowsPerSeries == null) {
    return {
      tone: "warning",
      titleKey: "forecast.config.messages.futureRowsPerSeriesMismatchTitle",
      bodyKey: "forecast.config.messages.futureRowsPerSeriesMismatchBody",
    };
  }
  if (profile.selectedCovariates > 0 && !supportsPast) {
    return {
      tone: "warning",
      titleKey: "forecast.config.messages.contextUnsupportedTitle",
      bodyKey: "forecast.config.messages.contextUnsupportedBody",
    };
  }
  if (profile.selectedCovariates > 0 && profile.futureRows > 0 && !supportsFuture) {
    return {
      tone: "warning",
      titleKey: "forecast.config.messages.futureContextUnsupportedTitle",
      bodyKey: "forecast.config.messages.futureContextUnsupportedBody",
    };
  }
  if (profile.selectedCovariates > 0 && profile.futureRows > 0) {
    if (profile.futureContextColumns.length > 0) {
      return {
        tone: "ok",
        titleKey: "forecast.config.messages.futureContextReadyTitle",
        bodyKey: "forecast.config.messages.futureContextReadyBody",
        values: { count: profile.futureContextColumns.length },
      };
    }
    return {
      tone: "info",
      titleKey: "forecast.config.messages.futureContextMissingTitle",
      bodyKey: "forecast.config.messages.futureContextMissingBody",
    };
  }
  if (profile.selectedCovariates > 0 && profile.futureRows === 0 && supportsFuture) {
    return {
      tone: "info",
      titleKey: "forecast.config.messages.addFutureRowsTitle",
      bodyKey: "forecast.config.messages.addFutureRowsBody",
    };
  }
  if (profile.futureRows > 0) {
    return {
      tone: "info",
      titleKey: "forecast.config.messages.futureDatesOnlyTitle",
      bodyKey: "forecast.config.messages.futureDatesOnlyBody",
    };
  }
  return null;
}
