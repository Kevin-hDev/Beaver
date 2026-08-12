import { useTranslation } from "react-i18next";
import type { WakeupRun } from "@/types/wakeup";
import { formatDateTime, formatRunStatus } from "@/lib/wakeup-format";
import { wakeupRunErrorMessage } from "@/lib/wakeup-run-error";
import { SettingsCard } from "@/components/settings/settings-card";

interface WakeupHistoryProps {
  runs: WakeupRun[];
}

export function WakeupHistory({ runs }: WakeupHistoryProps) {
  const { t } = useTranslation();
  const recent = runs.slice(0, 8);

  return (
    <section className="wk-history">
      <h3 className="wk-history-title">{t("heartbeat.history.title")}</h3>
      {recent.length === 0 ? (
        <SettingsCard>
          <div className="wk-history-empty">{t("heartbeat.history.empty")}</div>
        </SettingsCard>
      ) : (
        <SettingsCard>
          <div className="wk-history-list">
            {recent.map((run) => {
              const errorMessage = wakeupRunErrorMessage(run, t);
              return (
                <div className="wk-history-row" key={`${run.wakeup_id}-${run.fired_at}`}>
                  <span className={`wk-history-status wk-history-status-${run.status}`}>
                    {formatRunStatus(run.status)}
                  </span>
                  <span className="wk-history-time">{formatDateTime(run.fired_at)}</span>
                  {errorMessage && <span className="wk-history-error">{errorMessage}</span>}
                </div>
              );
            })}
          </div>
        </SettingsCard>
      )}
    </section>
  );
}
