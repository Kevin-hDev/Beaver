import "./context-progress.css";
import { useTranslation } from "react-i18next";
import type { ContextUsageBreakdown, ContextUsageItem } from "@/hooks/context-usage-breakdown";

interface ContextProgressProps {
  used: number;
  max: number;
  breakdown?: ContextUsageBreakdown;
}

type ColorKey = "neutral" | "yellow" | "orange" | "red";

function colorForPercentage(p: number): ColorKey {
  if (p >= 90) return "red";
  if (p >= 70) return "orange";
  if (p >= 55) return "yellow";
  return "neutral";
}

const FILL_COLORS: Record<ColorKey, string> = {
  neutral: "var(--context-ring-fill)",
  yellow: "var(--signal-warning)",
  orange: "var(--tool-bash)",
  red: "var(--signal-error)",
};

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

const SIZE = 16;
const STROKE = 3;
const RADIUS = (SIZE - STROKE) / 2;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

export function ContextProgress({ used, max, breakdown }: ContextProgressProps) {
  const { t } = useTranslation();
  if (!max || max <= 0) return null;
  const resolvedUsed = breakdown?.used ?? used;
  const percentage = Math.min((resolvedUsed / max) * 100, 100);
  const colorKey = colorForPercentage(percentage);
  const offset = CIRCUMFERENCE - (percentage / 100) * CIRCUMFERENCE;
  const pctDisplay = percentage < 1 ? "0" : percentage.toFixed(1);
  const items = breakdown?.items ?? [];

  return (
    <span className="context-ring">
      <button type="button" className="icon-btn context-ring-button" aria-label={t("agentLocal.contextUsage.title")}>
        <svg width={SIZE} height={SIZE} viewBox={`0 0 ${SIZE} ${SIZE}`}>
          <circle
            className="context-ring-track"
            cx={SIZE / 2}
            cy={SIZE / 2}
            r={RADIUS}
            strokeWidth={STROKE}
          />
          <circle
            className="context-ring-fill"
            cx={SIZE / 2}
            cy={SIZE / 2}
            r={RADIUS}
            strokeWidth={STROKE}
            stroke={FILL_COLORS[colorKey]}
            strokeDasharray={CIRCUMFERENCE}
            strokeDashoffset={offset}
          />
        </svg>
      </button>
      <div className="context-ring-panel" role="tooltip">
        <div className="context-ring-header">
          <span>{t("agentLocal.contextUsage.title")}</span>
          <strong>{formatTokens(resolvedUsed)} / {formatTokens(max)} ({pctDisplay}%)</strong>
        </div>
        <div className="context-ring-bar" aria-hidden="true">
          <div className="context-ring-bar-fill" style={{ width: `${percentage}%` }} />
        </div>
        <div className="context-ring-list">
          {items.map((item) => (
            <ContextUsageRow key={item.key} item={item} />
          ))}
        </div>
      </div>
    </span>
  );
}

function ContextUsageRow({ item }: { item: ContextUsageItem }) {
  const { t } = useTranslation();
  return (
    <div className="context-ring-row">
      <span className={`context-ring-dot context-ring-dot-${item.key}`} aria-hidden="true" />
      <span className="context-ring-label">{t(`agentLocal.contextUsage.categories.${item.key}`)}</span>
      <span className="context-ring-values">
        {formatTokens(item.tokens)}
        <span>{formatShare(item.percentage)}%</span>
      </span>
    </div>
  );
}

function formatShare(value: number): string {
  if (value > 0 && value < 0.1) return "<0.1";
  return value.toFixed(1);
}
