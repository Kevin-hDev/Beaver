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
  const differentBreakdown = breakdown
    && (summary.used === null || breakdown.used !== summary.used);
  return <>
    <div className="context-ring-header">
      <span>{t("agentLocal.contextUsage.title")}</span>
      <strong>{formatTotal(summary, percentage)}</strong>
    </div>
    <div className="context-ring-status">
      <span>{t(`agentLocal.contextUsage.${summary.status}`)}</span>
      {summary.secondaryStatus && <span>
        {t(`agentLocal.contextUsage.${summary.secondaryStatus}`)}
      </span>}
    </div>
    {summary.source && <div className="context-ring-source">
      {t(`agentLocal.contextUsage.source${sourceSuffix(summary.source)}`)}
    </div>}
    {summary.output !== null && <div className="context-ring-output">
      <span>{t("agentLocal.contextUsage.output")}</span>
      <strong>{formatTokenCount(summary.output)}</strong>
    </div>}
    {percentage !== null && <div className="context-ring-bar" aria-hidden="true">
      <div className="context-ring-bar-fill" style={{ width: `${percentage}%` }} />
    </div>}
    {breakdown && <>
      {differentBreakdown && <div className="context-ring-breakdown-label">
        {t("agentLocal.contextUsage.estimatedBreakdown")} · {formatTokenCount(breakdown.used)}
      </div>}
      <div className="context-ring-list">
        {breakdown.items.map((item) => <ContextUsageRow key={item.key} item={item} />)}
      </div>
    </>}
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

function sourceSuffix(source: NonNullable<ResolvedContextUsage["source"]>): string {
  return {
    provider: "Provider",
    native_counter: "Native",
    model_tokenizer: "Tokenizer",
    heuristic: "Heuristic",
    reconstructed: "Heuristic",
  }[source];
}
