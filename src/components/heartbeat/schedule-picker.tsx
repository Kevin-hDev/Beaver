import { useTranslation } from "react-i18next";
import type { WakeupSchedule } from "@/types/wakeup";

interface SchedulePickerProps {
  value: WakeupSchedule;
  onChange: (schedule: WakeupSchedule) => void;
}

export function SchedulePicker({ value, onChange }: SchedulePickerProps) {
  const { t } = useTranslation();

  const setKind = (kind: WakeupSchedule["kind"]) => {
    if (kind === value.kind) return;
    const now = new Date();
    const pad = (n: number) => n.toString().padStart(2, "0");
    const datetime = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
    switch (kind) {
      case "once":
        onChange({ kind: "once", datetime });
        break;
      case "daily":
        onChange({ kind: "daily", time: "08:00" });
        break;
      case "weekly":
        onChange({ kind: "weekly", weekday: 0, time: "08:00" });
        break;
    }
  };

  return (
    <div className="wk-form-field">
      <label className="wk-form-label">{t("heartbeat.form.schedule")}</label>
      <div className="wk-schedule-tabs">
        {(["once", "daily", "weekly"] as const).map((k) => (
          <button
            key={k}
            type="button"
            className={`wk-schedule-tab ${value.kind === k ? "active" : ""}`}
            onClick={() => setKind(k)}
          >
            {t(`heartbeat.form.scheduleKind.${k}`)}
          </button>
        ))}
      </div>

      {value.kind === "once" && (
        <input
          type="datetime-local"
          className="wk-input"
          value={value.datetime}
          onChange={(e) => onChange({ kind: "once", datetime: e.target.value })}
          required
        />
      )}

      {value.kind === "daily" && (
        <input
          type="time"
          className="wk-input"
          value={value.time}
          onChange={(e) => onChange({ kind: "daily", time: e.target.value })}
          required
        />
      )}

      {value.kind === "weekly" && (
        <div className="wk-schedule-row">
          <select
            className="wk-input"
            value={value.weekday}
            onChange={(e) =>
              onChange({ kind: "weekly", weekday: Number(e.target.value), time: value.time })
            }
          >
            {[0, 1, 2, 3, 4, 5, 6].map((d) => (
              <option key={d} value={d}>
                {t(`heartbeat.form.weekdays.${d}`)}
              </option>
            ))}
          </select>
          <input
            type="time"
            className="wk-input"
            value={value.time}
            onChange={(e) =>
              onChange({ kind: "weekly", weekday: value.weekday, time: e.target.value })
            }
            required
          />
        </div>
      )}
    </div>
  );
}
