import { useTranslation } from "react-i18next";
import type { WakeupRun } from "@/types/wakeup";
import { formatDateTime, formatRunStatus } from "@/lib/wakeup-format";
import { wakeupRunErrorMessage } from "@/lib/wakeup-run-error";
import { SettingsCard } from "@/components/settings/settings-card";

interface WakeupHistoryProps {
  runs: WakeupRun[];
  hasMore: boolean;
  onLoadMore: () => void;
}

export function WakeupHistory({ runs, hasMore, onLoadMore }: WakeupHistoryProps) {
  const { t } = useTranslation();

  return (
    <section className="wk-history">
      <h3 className="wk-history-title">{t("heartbeat.history.title")}</h3>
      {runs.length === 0 ? (
        <SettingsCard>
          <div className="wk-history-empty">{t("heartbeat.history.empty")}</div>
        </SettingsCard>
      ) : (
        <SettingsCard>
          <div className="wk-history-list">
            {runs.map((run) => {
              const errorMessage = wakeupRunErrorMessage(run, t);
              return (
                <div className="wk-history-row" key={run.run_id ?? `${run.automation_id}-${run.scheduled_for}`}>
                  <span className={`wk-history-status wk-history-status-${run.status}`}>
                    {formatRunStatus(run.status)}
                  </span>
                  <span className="wk-history-time">
                    {t("heartbeat.history.started")}: {formatDateTime(run.started_at)} · {t("heartbeat.history.finished")}: {formatDateTime(run.finished_at)}
                  </span>
                  {errorMessage && <span className="wk-history-error">{errorMessage}</span>}
                </div>
              );
            })}
          </div>
        </SettingsCard>
      )}
      {hasMore && <button className="btn btn-sm btn-secondary" type="button" onClick={onLoadMore}>{t("heartbeat.history.loadMore")}</button>}
    </section>
  );
}
