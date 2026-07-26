import { useTranslation } from "react-i18next";
import { Clock } from "@/components/ui/icons";
import type { WakeupSchedule } from "@/types/wakeup";
import { formatSchedule } from "@/lib/wakeup-format";

interface ActiveBadgeProps {
  active: boolean;
}

export function ActiveBadge({ active }: ActiveBadgeProps) {
  const { t } = useTranslation();
  const label = active ? t("heartbeat.badges.active") : t("heartbeat.badges.inactive");
  const className = active ? "wk-badge wk-badge-active" : "wk-badge wk-badge-inactive";
  return (
    <span className={className}>
      <Clock size="var(--icon-xs)" weight="regular" />
      {label}
    </span>
  );
}

interface ScheduleBadgeProps {
  schedule: WakeupSchedule;
}

export function ScheduleBadge({ schedule }: ScheduleBadgeProps) {
  return <span className="wk-badge wk-badge-schedule">{formatSchedule(schedule)}</span>;
}
