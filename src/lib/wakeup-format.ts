import type { WakeupRunStatus, WakeupSchedule } from "@/types/wakeup";
import i18n from "@/i18n";

function shortMonth(monthIndex: number): string {
  const date = new Date(2000, monthIndex);
  return new Intl.DateTimeFormat(i18n.language, { month: "short" }).format(date);
}

function shortWeekday(dayIndex: number): string {
  const date = new Date(2000, 0, 3 + dayIndex);
  return new Intl.DateTimeFormat(i18n.language, { weekday: "short" }).format(date);
}

function parseHM(time: string): { h: number; m: number } | null {
  const match = /^(\d{2}):(\d{2})$/.exec(time);
  if (!match) return null;
  return { h: parseInt(match[1], 10), m: parseInt(match[2], 10) };
}

function formatTime(h: number, m: number): string {
  return `${h.toString().padStart(2, "0")}h${m.toString().padStart(2, "0")}`;
}

export function formatSchedule(schedule: WakeupSchedule): string {
  switch (schedule.kind) {
    case "once": {
      const m = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})$/.exec(schedule.datetime);
      if (!m) return schedule.datetime;
      const month = shortMonth(parseInt(m[2], 10) - 1);
      return `${parseInt(m[3], 10)} ${month} · ${formatTime(parseInt(m[4], 10), parseInt(m[5], 10))}`;
    }
    case "daily": {
      const parsed = parseHM(schedule.time);
      if (!parsed) return schedule.time;
      return `${formatTime(parsed.h, parsed.m)} · ${i18n.t("wakeupFormat.daily")}`;
    }
    case "weekly": {
      const parsed = parseHM(schedule.time);
      const day = shortWeekday(schedule.weekday);
      if (!parsed) return day;
      return `${day} ${formatTime(parsed.h, parsed.m)} · ${i18n.t("wakeupFormat.weekly")}`;
    }
  }
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
