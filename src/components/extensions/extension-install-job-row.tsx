import { useTranslation } from "react-i18next";
import type { InstallJobView } from "@/types/extension-install-jobs.generated";
import { OperationProgressAction } from "@/components/ui/operation-progress-action";
import { formatByteSize } from "@/lib/format-byte-size";
import "./extension-install-job-row.css";

export interface ExtensionInstallActions {
  cancel: (id: string) => void;
  continue: (id: string, confirmationId: string) => void;
  resume: (id: string) => void;
  retry: (id: string) => void;
  dismiss: (id: string) => void;
  open: (extensionId: string) => void;
}
interface ExtensionInstallJobRowProps {
  job: InstallJobView;
  blockerName?: string;
  busy?: boolean;
  errorKey?: string | null;
  actions: ExtensionInstallActions;
  onShowRequest: (id: string) => void;
}

export function ExtensionInstallJobRow({
  job, blockerName, busy = false, errorKey, actions, onShowRequest,
}: ExtensionInstallJobRowProps) {
  const { t, i18n } = useTranslation();
  const key = "extensionInstalls";
  const working = job.status === "running" || job.status === "cancelling";
  const terminal = ["completed", "cancelled", "failed", "interrupted"].includes(job.status);
  const percent = job.downloadTotalBytes !== null && job.downloadTotalBytes > 0 && job.downloadedBytes !== null
    ? job.downloadedBytes / job.downloadTotalBytes * 100 : null;
  return (
    <article className="update-bubble eij-root" data-install-job={job.id}
      tabIndex={-1} aria-label={job.displayName}>
      <strong className="update-bubble-name">{job.displayName}</strong>
      <span className="eij-status" role="status">{t(`${key}.${working ? `phase.${job.phase}` : `status.${job.status}`}`)}</span>
      {working && <OperationProgressAction percent={percent} phaseLabel={t(`${key}.phase.${job.phase}`)}
        cancelling={job.status === "cancelling"} canCancel={job.canCancel && !busy}
        cancelLabel={t(`${key}.cancel`)} cancellingLabel={t(`${key}.status.cancelling`)}
        onCancel={() => actions.cancel(job.id)} />}
      {working && <span className="eij-status">
        {t(`${key}.occupiedSpace`, { size: formatByteSize(job.occupiedBytes, i18n.language) })}
      </span>}
      {job.queueBlocker && <div className="eij-message">
        <p>{t(`${key}.queueBlocked`, { name: blockerName ?? t(`${key}.anotherInstallation`) })}</p>
        <button type="button" className="btn btn-sm" onClick={() => onShowRequest(job.queueBlocker!.jobId)}>{t(`${key}.showRequest`)}</button>
      </div>}
      {job.status === "awaitingConfirmation" && <div className="eij-message">
        <p>{t(`${key}.volumeReason`, { size: formatByteSize(job.occupiedBytes, i18n.language) })}</p>
        {job.freeBytes !== null && <p>{t(`${key}.availableSpace`, { size: formatByteSize(job.freeBytes, i18n.language) })}</p>}
        <button type="button" className="btn btn-sm btn-primary" disabled={busy || !job.confirmationId}
          onClick={() => job.confirmationId && actions.continue(job.id, job.confirmationId)}>{t(`${key}.continue`)}</button>
      </div>}
      {job.status === "interrupted" && !job.canResume && <p className="eij-message">{t(`${key}.resumeUnavailable`)}</p>}
      {errorKey && <p className="eij-message" role="alert">{t(errorKey)}</p>}
      <div className="update-bubble-actions eij-actions">
        {!working && job.canCancel && <button type="button" className="btn btn-sm btn-destructive" disabled={busy}
          onClick={() => actions.cancel(job.id)}>{t(`${key}.cancel`)}</button>}
        {job.status === "interrupted" && job.canResume && <button type="button" className="btn btn-sm btn-primary" disabled={busy}
          onClick={() => actions.resume(job.id)}>{t(`${key}.resume`)}</button>}
        {(job.status === "failed" || job.status === "cancelled" || (job.status === "interrupted" && !job.canResume)) &&
          <button type="button" className="btn btn-sm" disabled={busy} onClick={() => actions.retry(job.id)}>{t(`${key}.retry`)}</button>}
        {job.status === "completed" && job.extensionId && <button type="button" className="btn btn-sm btn-primary"
          onClick={() => actions.open(job.extensionId!)}>{t(`${key}.open`)}</button>}
        {terminal && <button type="button" className="btn btn-sm" disabled={busy}
          onClick={() => actions.dismiss(job.id)}>{t(`${key}.dismiss`)}</button>}
      </div>
    </article>
  );
}
