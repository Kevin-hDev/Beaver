import { useTranslation } from "react-i18next";
import type { WakeupSchedule } from "@/types/wakeup";

interface SchedulePickerProps {
  value: WakeupSchedule;
  onChange: (schedule: WakeupSchedule) => void;
}

const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;

function localMinute(offsetMs: number): string {
  const date = new Date(Date.now() + offsetMs);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function defaultOnceSchedule(): WakeupSchedule {
  return { kind: "once", local_datetime: localMinute(86_400_000), timezone };
}

export function SchedulePicker({ value, onChange }: SchedulePickerProps) {
  const { t } = useTranslation();

  const setKind = (kind: WakeupSchedule["kind"]) => {
    if (kind === value.kind) return;
    switch (kind) {
      case "once":
        onChange({ kind: "once", local_datetime: localMinute(60_000), timezone });
        break;
      case "cron":
        onChange({ kind: "cron", expression: "0 8 * * *", timezone });
        break;
      case "after_completion":
        onChange({ kind: "after_completion", delay_minutes: 10 });
        break;
    }
  };

  return (
    <div className="nwd-field">
      <span className="nwd-label">{t("heartbeat.form.schedule")}</span>

      <div className="nwd-tabs" role="group">
        {(["once", "cron", "after_completion"] as const).map((k) => (
          <button
            key={k}
            type="button"
            className={`nwd-tab ${value.kind === k ? "is-active" : ""}`}
            aria-pressed={value.kind === k}
            onClick={() => setKind(k)}
          >
            {t(`heartbeat.form.scheduleKind.${k}`)}
          </button>
        ))}
      </div>

      <div className="nwd-when">
        {value.kind === "once" ? (
          <input
            type="datetime-local"
            className="field nwd-when-input"
            value={value.local_datetime}
            onChange={(e) => onChange({ ...value, local_datetime: e.target.value })}
            required
          />
        ) : value.kind === "cron" ? (
          <input
            type="text"
            className="field nwd-when-input"
            value={value.expression}
            onChange={(e) => onChange({ ...value, expression: e.target.value })}
            pattern="\S+\s+\S+\s+\S+\s+\S+\s+\S+"
            aria-label={t("heartbeat.form.cronExpression")}
            placeholder={t("heartbeat.form.cronPlaceholder")}
            required
          />
        ) : (
          <input
            type="number"
            className="field nwd-when-input"
            value={value.delay_minutes}
            min={1}
            max={525600}
            onChange={(e) => onChange({ kind: "after_completion", delay_minutes: Number(e.target.value) })}
            aria-label={t("heartbeat.form.delayMinutes")}
            required
          />
        )}
        {value.kind !== "after_completion" && <span className="nwd-when-label">{value.timezone}</span>}
      </div>
    </div>
  );
}
