import type { ScheduledWakeup, WakeupStatusSummary } from "@/types/wakeup";
import { formatDateTime } from "@/lib/wakeup-format";
import { ActiveBadge, ScheduleBadge } from "./badges";

interface WakeupCardProps {
  wakeup: ScheduledWakeup;
  summary?: WakeupStatusSummary;
  onClick: () => void;
}

export function WakeupCard({ wakeup, summary, onClick }: WakeupCardProps) {
  return (
    <button className="wk-card" onClick={onClick} type="button">
      <div className="wk-card-row wk-card-row-top">
        <span className="wk-card-model">{wakeup.model}</span>
        <span style={{ display: "flex", gap: 4, alignItems: "center" }}>
          {wakeup.provider !== "ollama" && (
            <span
              style={{
                fontSize: 10,
                padding: "2px 6px",
                background: "var(--surface)",
                borderRadius: 4,
                color: "var(--ink-muted)",
                textTransform: "uppercase",
                letterSpacing: "0.3px",
              }}
              title={wakeup.provider}
            >
              {wakeup.provider}
            </span>
          )}
          <ActiveBadge active={wakeup.active} />
        </span>
      </div>
      <div className="wk-card-row">
        <ScheduleBadge schedule={wakeup.schedule} />
      </div>
      <div className="wk-card-row wk-card-next">
        {formatDateTime(summary?.next_fire_at)}
      </div>
      <div className="wk-card-row wk-card-desc">
        {wakeup.description || wakeup.name}
      </div>
    </button>
  );
}
