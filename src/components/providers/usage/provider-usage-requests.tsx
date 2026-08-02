import { SettingsCard } from "@/components/settings/settings-card";
import type { ProviderUsageSnapshot } from "@/types/provider-usage";
import { useTranslation } from "react-i18next";
import { formatCount } from "./provider-usage-format";
import "./provider-usage-requests.css";

type RequestMetrics = ProviderUsageSnapshot["request_metrics"];

interface Props {
  metrics: RequestMetrics | null;
  loading: boolean;
}

export function ProviderUsageRequests({ metrics, loading }: Props) {
  const { t, i18n } = useTranslation();
  const latest = metrics && metrics.recent.length > 0
    ? metrics.recent[metrics.recent.length - 1]
    : undefined;
  const session = metrics?.sessions[0];
  const message = statusMessage(metrics, loading, t);

  return (
    <SettingsCard>
      {message ? (
        <div className="settings-row pur-empty"><span>{message}</span></div>
      ) : latest ? (
        <>
          <div className="settings-row pur-summary">
            <span>{t("providers.usage.requestMetrics.latestRequest")}</span>
            <strong>{latest.model}<small>{t(`providers.usage.requestMetrics.statuses.${latest.status}`)} · {latest.turn !== null && <>{t("providers.usage.requestMetrics.turn", { value: latest.turn })} · </>}{t("providers.usage.requestMetrics.attempt", { value: latest.attempt })}</small></strong>
          </div>
          {latest.routed_provider && latest.routed_model && (
            <div className="settings-row pur-row">
              <span>{t("providers.usage.requestMetrics.routedProvider")}</span>
              <strong>{latest.routed_provider}<small>{latest.routed_model}</small></strong>
            </div>
          )}
          <Timing label={t("providers.usage.requestMetrics.headers")} value={latest.timing.headers_ms} locale={i18n.language} />
          <Timing label={t("providers.usage.requestMetrics.firstEvent")} value={latest.timing.first_event_ms} locale={i18n.language} />
          <Timing label={t("providers.usage.requestMetrics.firstUseful")} value={latest.timing.first_useful_ms} locale={i18n.language} />
          <Timing label={t("providers.usage.requestMetrics.totalDuration")} value={latest.timing.total_ms} locale={i18n.language} />
          <Token label={t("providers.usage.cachedTokens")} value={latest.usage?.cached_input_tokens} locale={i18n.language} />
          <Token label={t("providers.usage.cacheWriteTokens")} value={latest.usage?.cache_write_input_tokens} locale={i18n.language} />
          <Token label={t("providers.usage.cacheMissTokens")} value={latest.usage?.cache_miss_input_tokens} locale={i18n.language} />
          {!latest.usage_complete && <div className="settings-row pur-notice"><span>{t("providers.usage.requestMetrics.usageIncomplete")}</span></div>}
          {session && (
            <>
              <div className="settings-row pur-session">
                <span>{t("providers.usage.requestMetrics.latestSession")}</span>
                <strong>{formatCount(session.attempt_count, i18n.language)} {t("providers.usage.requests").toLocaleLowerCase(i18n.language)}<small>{formatDuration(session.total_duration_ms, i18n.language)}</small></strong>
              </div>
              <Token label={t("providers.usage.cachedTokens")} value={observed(session.cache_read_tokens, session.cache_read_observation_count)} locale={i18n.language} />
              <Token label={t("providers.usage.cacheWriteTokens")} value={observed(session.cache_write_tokens, session.cache_write_observation_count)} locale={i18n.language} />
              <Token label={t("providers.usage.cacheMissTokens")} value={observed(session.cache_miss_tokens, session.cache_miss_observation_count)} locale={i18n.language} />
            </>
          )}
        </>
      ) : null}
    </SettingsCard>
  );
}

function statusMessage(
  metrics: RequestMetrics | null,
  loading: boolean,
  t: (key: string) => string,
): string | null {
  if (loading && !metrics) return t("common.loading");
  if (metrics?.availability === "unavailable") return t("providers.usage.requestMetrics.unavailable");
  if (!metrics || metrics.recent.length === 0) return t("providers.usage.requestMetrics.empty");
  return null;
}

function Timing({ label, value, locale }: { label: string; value: number | null; locale: string }) {
  return <div className="settings-row pur-row"><span>{label}</span><strong>{value == null ? "—" : formatDuration(value, locale)}</strong></div>;
}

function Token({ label, value, locale }: { label: string; value: number | null | undefined; locale: string }) {
  return <div className="settings-row pur-row"><span>{label}</span><strong>{value == null ? "—" : formatCount(value, locale)}</strong></div>;
}

function observed(value: number, observationCount: number): number | undefined {
  return observationCount > 0 ? value : undefined;
}

function formatDuration(milliseconds: number, locale: string): string {
  if (milliseconds < 1_000) return `${formatCount(milliseconds, locale)} ms`;
  return `${new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(milliseconds / 1_000)} s`;
}
