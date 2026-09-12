import { useTranslation } from "react-i18next";
import type { ContextUsageBreakdown } from "@/hooks/context-usage-breakdown";
import type { ResolvedContextUsage } from "@/hooks/agent-token-estimate";
import type { ResolvedCompressionProfileView } from "@/types/compression-profile.generated";
import { formatTokenCount } from "@/lib/token-format";
import { ContextCompressionHelpPopover } from "./context-compression-help-popover";
import { ContextUsageRow } from "./context-progress-row";

interface ContextProgressPanelProps {
  summary: ResolvedContextUsage;
  breakdown?: ContextUsageBreakdown;
  compression?: ResolvedCompressionProfileView | null;
  percentage: number | null;
  onCompressionHelpOpen: (open: boolean) => void;
}

export function ContextProgressPanel({
  summary, breakdown, compression, percentage, onCompressionHelpOpen,
}: ContextProgressPanelProps) {
  const { t } = useTranslation();
  return <>
    <div className="context-ring-header">
      <span>{t("agentLocal.contextUsage.title")}</span>
      <strong>{formatTotal(summary, percentage)}</strong>
    </div>
    {percentage !== null && <div className="context-ring-bar" aria-hidden="true">
      <div className="context-ring-bar-fill" style={{ width: `${percentage}%` }} />
    </div>}
    {breakdown && <div className="context-ring-list">
      {breakdown.items.map((item) => <ContextUsageRow key={item.key} item={item} />)}
    </div>}
    <div className="context-ring-compression-row">
      {compression ? compression.available ? <>
        <span>{t("agentLocal.contextUsage.compression")}</span>
        <strong title={compression.name}>{compression.name}</strong>
      </> : <>
        <span>{t("agentLocal.contextUsage.compressionDisabled")}</span>
        <ContextCompressionHelpPopover onOpenChange={onCompressionHelpOpen} />
      </> : <>
        <span>{t("agentLocal.contextUsage.compression")}</span>
        <strong aria-busy="true">—</strong>
      </>}
    </div>
  </>;
}

function formatTotal(summary: ResolvedContextUsage, percentage: number | null): string {
  if (summary.used === null) return "—";
  const used = formatTokenCount(summary.used);
  if (summary.max === null || percentage === null) return used;
  const display = percentage < 1 ? "0" : percentage.toFixed(1);
  return `${used} / ${formatTokenCount(summary.max)} (${display}%)`;
}
