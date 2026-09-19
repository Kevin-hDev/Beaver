import { useTranslation } from "react-i18next";
import beaverUrl from "../../../assets/QPXAq01-anime.svg";
import { OperationProgressBar } from "@/components/ui/operation-progress-action";
import { CheckCircle2, Clock3, Warning, X } from "@/components/ui/icons";
import type { UpdateOperationSnapshot } from "@/types/update-progress.generated";
import { cn } from "@/lib/utils";
import { formatByteSize } from "@/lib/format-byte-size";
import { cancelUpdateOperation } from "./update-window-actions";
import "./update-operation-row.css";

export function UpdateOperationRow({ operation, onDismiss, onRetry }: {
  operation: UpdateOperationSnapshot;
  onDismiss: (id: string) => Promise<void>;
  onRetry: (id: string) => Promise<void>;
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
          {operation.kind === "voice-model" && operation.label === "silero-vad"
            ? t("voice.settings.title")
            : operation.label}
        </span>
        <OperationState operation={operation} phaseLabel={phaseLabel} />
      </div>
      <div className="upw-actions">
        {operation.canCancel && (
          <button type="button" className="btn btn-sm btn-secondary"
            onClick={() => void cancelUpdateOperation(operation).catch(() => {})}>
            {t("updates.window.cancel")}
          </button>
        )}
        {operation.canRetry && (
          <button type="button" className="btn btn-sm btn-secondary"
            onClick={() => void onRetry(operation.id).catch(() => {})}>
            {t("updates.window.retry")}
          </button>
        )}
        {terminal && (
          <button type="button" className="icon-btn" aria-label={t("updates.window.remove")}
            onClick={() => void onDismiss(operation.id).catch(() => {})}>
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
  const { t, i18n } = useTranslation();
  if (operation.status === "queued") {
    return <span className="upw-rank"><Clock3 aria-hidden="true" />{t("updates.window.queued", { position: operation.queuePosition })}</span>;
  }
  if (operation.status === "cancelling") {
    return <span className="upw-step upw-step-transient">{t("updates.window.cancelling")}</span>;
  }
  if (operation.status === "failed") {
    if (operation.missingBytes !== null) {
      return <span className="upw-failure">{t("updates.window.insufficientSpace", {
        size: formatByteSize(operation.missingBytes, i18n.language),
      })}</span>;
    }
    return <span className="upw-failure">{t(operation.isUpdate === false ? "updates.window.downloadFailed" : "updates.window.failed")}</span>;
  }
  if (operation.status === "completed" || operation.status === "cancelled") {
    const resultKey = operation.status === "cancelled"
      ? "updates.window.cancelledResult"
      : operation.isUpdate === false ? "updates.window.downloadCompleted" : "updates.window.completed";
    return <span className="upw-finished"><CheckCircle2 aria-hidden="true" />{t(resultKey)}</span>;
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
