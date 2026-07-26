import { SettingsCard } from "@/components/settings/settings-card";
import { ArrowSquareOut } from "@/components/ui/icons";
import { open } from "@tauri-apps/plugin-shell";
import { useTranslation } from "react-i18next";
import type { ProviderUsageSnapshot } from "@/types/provider-usage";
import { formatBalance, formatCount, formatDate } from "./provider-usage-format";

interface Props {
  snapshot: ProviderUsageSnapshot | null;
  loading: boolean;
  siteUrl: string;
}

export function ProviderUsageLimits({ snapshot, loading, siteUrl }: Props) {
  const { t, i18n } = useTranslation();
  if (loading && !snapshot) return <SettingsCard><StatusRow value={t("common.loading")} /></SettingsCard>;

  const hasRemoteData = Boolean(snapshot?.windows.length || snapshot?.balances.length);
  const groups = groupWindows(snapshot?.windows ?? []);
  const showRemaining = snapshot?.auth_source === "oauth";
  const windowRows = (windows: UsageWindow[]) => windows.map((window, index) => {
    const progress = progressValue(window, showRemaining);
    return (
      <div className="settings-row puc-window" key={`${window.label_code}-${index}`}>
        <div className="puc-row">
          <span>{t(`providers.usage.windows.${window.label_code}`, window.label_code)}</span>
          <strong>{windowValue(window, showRemaining, i18n.language, t)}</strong>
        </div>
        {progress !== null && window.limit !== null && (
          <div
            className="puc-progress"
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={Math.round(progress)}
          >
            <span style={{ width: `${progress}%` }} />
          </div>
        )}
        {window.resets_at && (
          <div className="puc-reset">
            {t("providers.usage.resetsAt", { date: formatDate(window.resets_at, i18n.language) })}
          </div>
        )}
      </div>
    );
  });

  if (groups) {
    return (
      <div className="puc-limit-groups">
        {groups.map((group) => (
          <section className="puc-limit-group" key={group.code}>
            <h4>
              {group.name
                ? t("providers.usage.namedLimitsTitle", { name: group.name })
                : t("providers.usage.generalLimitsTitle")}
            </h4>
            <SettingsCard>{windowRows(group.windows)}</SettingsCard>
          </section>
        ))}
        <SettingsCard>
          <DetailRows snapshot={snapshot} siteUrl={siteUrl} />
        </SettingsCard>
      </div>
    );
  }

  return (
    <SettingsCard>
      {!hasRemoteData && <StatusRow value={t("providers.usage.remoteUnavailable")} />}
      {windowRows(snapshot?.windows ?? [])}
      <DetailRows snapshot={snapshot} siteUrl={siteUrl} />
    </SettingsCard>
  );
}

function DetailRows({ snapshot, siteUrl }: Pick<Props, "snapshot" | "siteUrl">) {
  const { t, i18n } = useTranslation();
  return (
    <>
      {snapshot?.balances.map((balance, index) => (
        <div className="settings-row puc-row" key={`${balance.label_code}-${index}`}>
          <span>{t(`providers.usage.balances.${balance.label_code}`, balance.label_code)}</span>
          <strong>{formatBalance(balance.amount, balance.currency, i18n.language)}</strong>
        </div>
      ))}
      {snapshot?.notice_code && (
        <div className="settings-row puc-notice" role="status">
          {t(`providers.usage.notices.${snapshot.notice_code}`, snapshot.notice_code)}
        </div>
      )}
      <div className="settings-row puc-row">
        <span>
          {snapshot?.stale ? t("providers.usage.stale") : t("providers.usage.updated")}
          {snapshot?.refreshed_at ? <small>{formatDate(snapshot.refreshed_at, i18n.language)}</small> : null}
        </span>
        <button type="button" className="puc-site" onClick={() => void open(siteUrl)}>
          {t("providers.usage.openProviderSite")} <ArrowSquareOut size="var(--icon-xs)" />
        </button>
      </div>
    </>
  );
}

function StatusRow({ value }: { value: string }) {
  return <div className="settings-row puc-row"><span>{value}</span></div>;
}

type UsageWindow = ProviderUsageSnapshot["windows"][number];

interface WindowGroup {
  code: string;
  name: string | null;
  windows: UsageWindow[];
}

function groupWindows(windows: UsageWindow[]): WindowGroup[] | null {
  if (!windows.some((window) => window.group_code)) return null;
  const groups: WindowGroup[] = [];
  for (const window of windows) {
    const code = window.group_code ?? "general";
    const existing = groups.find((group) => group.code === code);
    if (existing) {
      existing.windows.push(window);
    } else {
      groups.push({ code, name: window.group_name ?? null, windows: [window] });
    }
  }
  return groups;
}

function windowValue(
  window: UsageWindow,
  showRemaining: boolean,
  locale: string,
  t: (key: string, values?: Record<string, string>) => string,
): string {
  const remaining = remainingPercent(window);
  if (showRemaining && remaining !== null) {
    return t("providers.usage.remainingPercent", { value: formatCount(remaining, locale) });
  }
  if (window.used !== null && window.limit !== null) {
    return `${formatCount(window.used, locale)} / ${formatCount(window.limit, locale)}`;
  }
  if (window.remaining !== null) return formatCount(window.remaining, locale);
  if (window.used !== null) return formatCount(window.used, locale);
  return "—";
}

function progressValue(window: UsageWindow, showRemaining: boolean): number | null {
  const percent = showRemaining ? remainingPercent(window) : window.used_percent;
  return percent === null ? null : Math.min(100, Math.max(0, percent));
}

function remainingPercent(window: UsageWindow): number | null {
  let percent: number | null;
  if (window.remaining !== null && window.limit !== null && window.limit > 0) {
    percent = window.remaining / window.limit * 100;
  } else {
    percent = window.used_percent === null ? null : 100 - window.used_percent;
  }
  return percent === null ? null : Math.min(100, Math.max(0, percent));
}
