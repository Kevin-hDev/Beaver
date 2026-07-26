import { useTranslation } from "react-i18next";
import { Pulse, Plus } from "@/components/ui/icons";
import type { ScheduledWakeup, WakeupStatusSummary } from "@/types/wakeup";
import { WakeupCard } from "./wakeup-card";

interface WakeupGridProps {
  wakeups: ScheduledWakeup[];
  summaries: Record<string, WakeupStatusSummary>;
  onSelect: (id: string) => void;
  onCreate: () => void;
}

export function WakeupGrid({ wakeups, summaries, onSelect, onCreate }: WakeupGridProps) {
  const { t } = useTranslation();

  return (
    <div className="wk-main">
      <div className="wk-header">
        <div className="wk-header-title">
          <Pulse size="var(--icon-xl)" weight="regular" />
          <span>{t("heartbeat.title")}</span>
        </div>
        <div className="wk-header-subtitle">{t("heartbeat.subtitle")}</div>
        <button className="wk-new-btn" onClick={onCreate} type="button">
          <Plus size="var(--icon-sm)" weight="bold" />
          {t("heartbeat.newWakeup")}
        </button>
      </div>

      {wakeups.length === 0 ? (
        <div className="wk-empty">{t("heartbeat.empty")}</div>
      ) : (
        <div className="wk-grid">
          {wakeups.map((w) => (
            <WakeupCard
              key={w.id}
              wakeup={w}
              summary={summaries[w.id]}
              onClick={() => onSelect(w.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}
