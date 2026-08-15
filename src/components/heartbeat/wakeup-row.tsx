import type { ScheduledWakeup, WakeupStatusSummary } from "@/types/wakeup";
import { formatDateTime } from "@/lib/wakeup-format";
import { ActiveBadge, ScheduleBadge } from "./badges";

interface WakeupRowProps {
  wakeup: ScheduledWakeup;
  summary?: WakeupStatusSummary;
  onClick: () => void;
}

export function WakeupRow({ wakeup, summary, onClick }: WakeupRowProps) {
  return (
    <button className="wk-row" onClick={onClick} type="button">
      <span className="wk-row-info">
        <span className="wk-row-heading">
          <span className="wk-row-model">{wakeup.model}</span>
          {/* Ollama est le fournisseur par défaut : le nommer sur chaque ligne
              n'apprend rien et noie les autres. */}
          {wakeup.provider !== "ollama" && (
            <span className="wk-provider-tag" title={wakeup.provider}>{wakeup.provider}</span>
          )}
        </span>
        <span className="wk-row-desc">{wakeup.description || wakeup.name}</span>
        <span className="wk-row-next">{formatDateTime(summary?.next_fire_at)}</span>
      </span>
      <span className="wk-row-badges">
        <ScheduleBadge schedule={wakeup.schedule} />
        <ActiveBadge active={wakeup.active} />
      </span>
    </button>
  );
}
