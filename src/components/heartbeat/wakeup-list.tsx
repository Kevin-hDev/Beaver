import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { WakeupClockIcon } from "@/components/ui/wakeup-clock-icon";
import { SettingsCard } from "@/components/settings/settings-card";
import type { ScheduledWakeup, WakeupStatusSummary } from "@/types/wakeup";
import { WakeupRow } from "./wakeup-row";

interface WakeupListProps {
  wakeups: ScheduledWakeup[];
  summaries: Record<string, WakeupStatusSummary>;
  onSelect: (id: string) => void;
  onCreate: () => void;
}

export function WakeupList({ wakeups, summaries, onSelect, onCreate }: WakeupListProps) {
  const { t } = useTranslation();

  return (
    <div className="wk-main">
      <div className="wk-inner">
        <div className="wk-header">
          <div className="wk-header-title">
            <WakeupClockIcon />
            <span>{t("heartbeat.title")}</span>
          </div>
          <div className="wk-header-subtitle">{t("heartbeat.subtitle")}</div>
          <button className="btn btn-sm btn-secondary wk-new-btn" onClick={onCreate} type="button">
            <Plus size="var(--icon-sm)" weight="bold" />
            {t("heartbeat.newWakeup")}
          </button>
        </div>

        {wakeups.length === 0 ? (
          <SettingsCard>
            <div className="wk-empty">{t("heartbeat.empty")}</div>
          </SettingsCard>
        ) : (
          <SettingsCard className="settings-card-list">
            {wakeups.map((w) => (
              <WakeupRow
                key={w.id}
                wakeup={w}
                summary={summaries[w.id]}
                onClick={() => onSelect(w.id)}
              />
            ))}
          </SettingsCard>
        )}
      </div>
    </div>
  );
}
