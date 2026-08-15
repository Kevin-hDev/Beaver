import { useTranslation } from "react-i18next";
import { CustomSelect } from "@/components/ui/custom-select";
import type { WakeupSchedule } from "@/types/wakeup";

interface SchedulePickerProps {
  value: WakeupSchedule;
  onChange: (schedule: WakeupSchedule) => void;
}

const WEEKDAYS = [0, 1, 2, 3, 4, 5, 6] as const;

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
    <div className="nwd-field">
      <span className="nwd-label">{t("heartbeat.form.schedule")}</span>

      <div className="nwd-tabs" role="group">
        {(["once", "daily", "weekly"] as const).map((k) => (
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

      {/* Le moment se lit comme une phrase : « À 08:00 », « Lundi à 08:00 ».
          Ces champs occupent leur largeur utile, pas toute la ligne. */}
      <div className="nwd-when">
        {value.kind === "weekly" && (
          <div className="nwd-weekday">
            <CustomSelect
              value={String(value.weekday)}
              onChange={(day) => onChange({ kind: "weekly", weekday: Number(day), time: value.time })}
              options={WEEKDAYS.map((d) => ({ value: String(d), label: t(`heartbeat.form.weekdays.${d}`) }))}
            />
          </div>
        )}

        <span className="nwd-when-label">{t("heartbeat.form.at")}</span>

        {value.kind === "once" ? (
          <input
            type="datetime-local"
            className="field nwd-when-input"
            value={value.datetime}
            onChange={(e) => onChange({ kind: "once", datetime: e.target.value })}
            required
          />
        ) : (
          <input
            type="time"
            className="field nwd-when-input"
            value={value.time}
            onChange={(e) => onChange(
              value.kind === "weekly"
                ? { kind: "weekly", weekday: value.weekday, time: e.target.value }
                : { kind: "daily", time: e.target.value },
            )}
            required
          />
        )}
      </div>
    </div>
  );
}
