import { useState } from "react";
import { useTranslation } from "react-i18next";
import { SettingsCard } from "@/components/settings/settings-card";
import type { UsageAggregate, UsagePeriod, UsagePeriodId } from "@/types/provider-usage";
import { formatCount, formatUsdMicros } from "./provider-usage-format";

const PERIODS: UsagePeriodId[] = ["today", "seven_days", "thirty_days", "all_time"];

interface Props {
  periods: UsagePeriod[];
  loading: boolean;
}

export function ProviderUsageLocal({ periods, loading }: Props) {
  const { t, i18n } = useTranslation();
  const [selected, setSelected] = useState<UsagePeriodId>("seven_days");
  const period = periods.find((item) => item.period === selected);
  return (
    <SettingsCard>
      <div className="puc-periods" role="group" aria-label={t("providers.usage.periodLabel")}>
        {PERIODS.map((id) => (
          <button
            type="button"
            key={id}
            className={selected === id ? "active" : undefined}
            aria-pressed={selected === id}
            onClick={() => setSelected(id)}
          >
            {t(`providers.usage.periods.${id}`)}
          </button>
        ))}
      </div>
      {!period ? <div className="settings-row puc-row"><span>{loading ? t("common.loading") : t("providers.usage.historyEmpty")}</span></div> : (
        <>
          <Metric label={t("providers.usage.requests")} value={formatCount(period.totals.request_count, i18n.language)} />
          <Metric label={t("providers.usage.inputTokens")} value={tokenValue(period, period.totals.tokens.input_tokens, i18n.language)} badge={tokenBadge(period, t)} />
          <Metric label={t("providers.usage.outputTokens")} value={tokenValue(period, period.totals.tokens.output_tokens, i18n.language)} badge={tokenBadge(period, t)} />
          <Metric label={t("providers.usage.cachedTokens")} value={tokenValue(period, period.totals.tokens.cached_input_tokens, i18n.language)} badge={tokenBadge(period, t)} />
          <Metric label={t("providers.usage.reasoningTokens")} value={tokenValue(period, period.totals.tokens.reasoning_output_tokens, i18n.language)} badge={tokenBadge(period, t)} />
          <Metric label={t("providers.usage.totalTokens")} value={tokenValue(period, period.totals.tokens.total_tokens, i18n.language)} badge={tokenBadge(period, t)} />
          <Metric label={t("providers.usage.cost")} value={period.totals.priced_request_count > 0 ? formatUsdMicros(period.totals.cost_usd_micros, i18n.language) : "—"} badge={t(`providers.usage.quality.${period.cost_quality}`)} />
          <Breakdown title={t("providers.usage.byOrigin")} values={[
            [t("providers.usage.origins.manual_chat"), period.origins.manual_chat],
            [t("providers.usage.origins.external_channel"), period.origins.external_channel],
            [t("providers.usage.origins.automation"), period.origins.automation],
          ]} />
          <Breakdown title={t("providers.usage.byWorkload")} values={[
            [t("providers.usage.workloads.primary"), period.workloads.primary],
            [t("providers.usage.workloads.subagent"), period.workloads.subagent],
            [t("providers.usage.workloads.compression"), period.workloads.compression],
          ]} />
        </>
      )}
    </SettingsCard>
  );
}

function Metric({ label, value, badge }: { label: string; value: string; badge?: string }) {
  return <div className="settings-row puc-row"><span>{label}</span><strong>{value}{badge && <small>{badge}</small>}</strong></div>;
}

function Breakdown({ title, values }: { title: string; values: Array<[string, UsageAggregate]> }) {
  const { i18n } = useTranslation();
  return <div className="settings-row puc-breakdown"><span>{title}</span><div>{values.map(([label, value]) => (
    <span key={label}>{label}<strong>{formatCount(value.request_count, i18n.language)}</strong></span>
  ))}</div></div>;
}

function tokenValue(period: UsagePeriod, value: number, locale: string): string {
  return period.totals.usage_request_count > 0 ? formatCount(value, locale) : "—";
}

function tokenBadge(period: UsagePeriod, t: (key: string) => string): string | undefined {
  if (period.totals.usage_request_count === 0) return t("providers.usage.quality.unavailable");
  if (period.totals.usage_request_count < period.totals.request_count) return t("providers.usage.quality.partial");
  return undefined;
}
