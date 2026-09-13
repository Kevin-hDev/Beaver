import { useTranslation } from "react-i18next";
import beaverUrl from "../../../../docs/fonctionnalites/install-&-update/assets/QPXAq01-anime.svg";
import { OperationProgressBar } from "@/components/ui/operation-progress-action";
import { CheckCircle2, Clock3, Warning, X } from "@/components/ui/icons";
import type { UpdateOperationSnapshot } from "@/types/update-progress.generated";
import { cn } from "@/lib/utils";
import { cancelUpdateOperation, retryUpdateOperation } from "./update-window-actions";
import "./update-operation-row.css";

export function UpdateOperationRow({ operation, onDismiss }: {
  operation: UpdateOperationSnapshot;
  onDismiss: (id: string) => Promise<void>;
}) {
  const { t } = useTranslation();
  const terminal = operation.status === "completed" || operation.status === "failed"
    || operation.status === "cancelled";
  const suspended = terminal || operation.status === "queued";
  const phaseLabel = t(`updates.window.phases.${operation.phase}`);

  return (
    <li className={cn("upw-line", suspended && "upw-line-suspended")}>
      <UpdateBeaver frozen={suspended} muted={operation.status === "queued" || operation.status === "failed" || operation.status === "cancelled"} />
      <div className="upw-body">
        <span className="upw-name">
          {operation.status === "failed" && <Warning className="upw-state-icon upw-error-icon" aria-hidden="true" />}
          {operation.label}
        </span>
        <OperationState operation={operation} phaseLabel={phaseLabel} />
      </div>
      <div className="upw-actions">
        {operation.canCancel && (
          <button type="button" className="btn btn-sm btn-secondary"
            onClick={() => void cancelUpdateOperation(operation)}>
            {t("updates.window.cancel")}
          </button>
        )}
        {operation.canRetry && (
          <button type="button" className="btn btn-sm btn-secondary"
            onClick={() => void retryUpdateOperation(operation.id)}>
            {t("updates.window.retry")}
          </button>
        )}
        {terminal && (
          <button type="button" className="icon-btn" aria-label={t("updates.window.remove")}
            onClick={() => void onDismiss(operation.id)}>
            <X size="var(--icon-sm)" />
          </button>
        )}
      </div>
    </li>
  );
}

function OperationState({ operation, phaseLabel }: {
  operation: UpdateOperationSnapshot;
  phaseLabel: string;
}) {
  const { t } = useTranslation();
  if (operation.status === "queued") {
    return <span className="upw-rank"><Clock3 aria-hidden="true" />{t("updates.window.queued", { position: operation.queuePosition })}</span>;
  }
  if (operation.status === "cancelling") {
    return <span className="upw-step upw-step-transient">{t("updates.window.cancelling")}</span>;
  }
  if (operation.status === "failed") return <span className="upw-failure">{t("updates.window.failed")}</span>;
  if (operation.status === "completed" || operation.status === "cancelled") {
    return <span className="upw-finished"><CheckCircle2 aria-hidden="true" />{t(operation.status === "completed" ? "updates.window.completed" : "updates.window.cancelledResult")}</span>;
  }
  if (operation.kind === "app-release" && operation.phase === "restarting") {
    return <span className="upw-restarting">{t("updates.window.restarting")}</span>;
  }
  return (
    <>
      <span className="upw-step">{phaseLabel}</span>
      {operation.progressMode !== "none" && (
        <div className="upw-measure">
          <OperationProgressBar
            percent={operation.progressMode === "determinate" ? operation.percent : null}
            phaseLabel={phaseLabel}
            cancelling={false}
          />
        </div>
      )}
    </>
  );
}

function UpdateBeaver({ frozen, muted }: { frozen: boolean; muted: boolean }) {
  return (
    <span className={cn("upw-beaver", muted && "upw-beaver-muted")} aria-hidden="true">
      {frozen ? (
        <svg viewBox="0 0 18000 18000">
          <g transform="translate(0,18000) scale(1,-1)">
            <use className="upw-beaver-surface" href={`${beaverUrl}#qpxa-surface`} />
            <use className="upw-beaver-ink" href={`${beaverUrl}#qpxa-encre`} />
          </g>
        </svg>
      ) : <img src={beaverUrl} alt="" />}
    </span>
  );
}
