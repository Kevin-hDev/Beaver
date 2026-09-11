import type { ScheduledWakeup } from "@/types/wakeup";
import { displayStatus } from "@/types/wakeup";
import { formatDateTime, formatRunStatus, formatTarget } from "@/lib/wakeup-format";
import { RunErrorBadge, ScheduleBadge, StatusBadge } from "./badges";

interface WakeupRowProps {
  wakeup: ScheduledWakeup;
  onClick: () => void;
}

export function WakeupRow({ wakeup, onClick }: WakeupRowProps) {
  return (
    <button className="wk-row" onClick={onClick} type="button">
      <span className="wk-row-info">
        <span className="wk-row-heading">
          <span className="wk-row-model">{wakeup.name}</span>
          <span className="wk-provider-tag" title={wakeup.provider}>{wakeup.provider}</span>
        </span>
        <span className="wk-row-desc">{wakeup.model} · {formatTarget(wakeup.target)}</span>
        <span className="wk-row-next">{formatDateTime(wakeup.next_fire_at)} · {formatRunStatus(wakeup.last_run?.status)}</span>
      </span>
      <span className="wk-row-badges">
        <ScheduleBadge schedule={wakeup.schedule} />
        <RunErrorBadge run={wakeup.last_run} />
        <StatusBadge status={displayStatus(wakeup)} />
      </span>
    </button>
  );
}
