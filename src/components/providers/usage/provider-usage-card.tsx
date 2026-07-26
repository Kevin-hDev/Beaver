import { ArrowsClockwise } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { useProviderUsage } from "@/hooks/use-provider-usage";
import { useTranslation } from "react-i18next";
import { ProviderUsageLimits } from "./provider-usage-limits";
import { ProviderUsageLocal } from "./provider-usage-local";
import "./provider-usage-card.css";

interface Props {
  connectionId: string;
  siteUrl: string;
}

export function ProviderUsageCard({ connectionId, siteUrl }: Props) {
  const { t } = useTranslation();
  const { snapshot, loading, refreshing, refresh } = useProviderUsage(connectionId);
  const current = snapshot?.connection_id === connectionId ? snapshot : null;

  return (
    <div className="puc-root">
      <section className="puc-section" aria-labelledby="puc-provider-title">
        <div className="puc-heading">
          <h3 id="puc-provider-title">{t("providers.usage.providerTitle")}</h3>
          <Tooltip label={t("providers.usage.refresh")} align="right">
            <button
              type="button"
              className="puc-refresh"
              aria-label={t("providers.usage.refresh")}
              disabled={refreshing}
              onClick={() => void refresh()}
            >
              <ArrowsClockwise
                size="var(--icon-sm)"
                className={refreshing ? "puc-spin" : undefined}
              />
            </button>
          </Tooltip>
        </div>
        <ProviderUsageLimits snapshot={current} loading={loading} siteUrl={siteUrl} />
      </section>
      <section className="puc-section" aria-labelledby="puc-local-title">
        <div className="puc-heading">
          <h3 id="puc-local-title">{t("providers.usage.localTitle")}</h3>
        </div>
        <ProviderUsageLocal periods={current?.local_periods ?? []} loading={loading} />
      </section>
    </div>
  );
}
