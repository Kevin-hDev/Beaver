import { useTranslation } from "react-i18next";
import type { ContextUsageItem } from "@/hooks/context-usage-breakdown";
import { formatTokenCount } from "@/lib/token-format";

export function ContextUsageRow({ item }: { item: ContextUsageItem }) {
  const { t } = useTranslation();
  return (
    <div className="context-ring-row">
      <span className={`context-ring-dot context-ring-dot-${item.key}`} aria-hidden="true" />
      <span className="context-ring-label">
        {t(`agentLocal.contextUsage.categories.${item.key}`)}
      </span>
      <span className="context-ring-values">
        {formatTokenCount(item.tokens)}
        <span>{formatShare(item.percentage)}%</span>
      </span>
    </div>
  );
}

function formatShare(value: number): string {
  if (value > 0 && value < 0.1) return "<0.1";
  return value.toFixed(1);
}
