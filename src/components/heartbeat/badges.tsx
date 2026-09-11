import { useTranslation } from "react-i18next";
import { Clock } from "@/components/ui/icons";
import type { WakeupDisplayStatus, WakeupLastRun, WakeupSchedule } from "@/types/wakeup";
import { formatSchedule } from "@/lib/wakeup-format";
import { wakeupRunErrorMessage } from "@/lib/wakeup-run-error";

interface StatusBadgeProps {
  status: WakeupDisplayStatus;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const { t } = useTranslation();
  return (
    <span className={`wk-badge wk-badge-${status}`}>
      <Clock size="var(--icon-xs)" weight="regular" />
      {t(`heartbeat.badges.${status}`)}
    </span>
  );
}

interface ScheduleBadgeProps {
  schedule: WakeupSchedule;
}

export function ScheduleBadge({ schedule }: ScheduleBadgeProps) {
  return <span className="wk-badge wk-badge-schedule">{formatSchedule(schedule)}</span>;
}

export function RunErrorBadge({ run }: { run: WakeupLastRun | null }) {
  const { t } = useTranslation();
  if (!run || (run.status !== "error" && run.status !== "missed" && run.status !== "interrupted")) return null;
  const message = wakeupRunErrorMessage({ error_code: run.error_code ?? undefined }, t);
  return <span className="wk-badge wk-badge-error">{message || t(`heartbeat.status.${run.status}`)}</span>;
}
