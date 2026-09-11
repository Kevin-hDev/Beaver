import type { WakeupRunStatus, WakeupSchedule, WakeupTarget } from "@/types/wakeup";
import i18n from "@/i18n";

export function formatSchedule(schedule: WakeupSchedule): string {
  switch (schedule.kind) {
    case "once": {
      return `${schedule.local_datetime.replace("T", " ")} · ${schedule.timezone}`;
    }
    case "cron": {
      return `${schedule.expression} · ${schedule.timezone}`;
    }
    case "after_completion": {
      return i18n.t("wakeupFormat.afterCompletion", { count: schedule.delay_minutes });
    }
  }
}

export function formatTarget(target: WakeupTarget): string {
  return i18n.t(`heartbeat.target.${target.mode}`);
}

export function formatDateTime(value: string | null | undefined): string {
  if (!value) return i18n.t("heartbeat.status.none");
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return i18n.t("heartbeat.status.none");
  return new Intl.DateTimeFormat(i18n.language, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

export function formatRunStatus(status: WakeupRunStatus | null | undefined): string {
  if (!status) return i18n.t("heartbeat.status.never");
  return i18n.t(`heartbeat.status.${status}`);
}
