import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { useExtensionInstallJobs } from "@/hooks/use-extension-install-jobs";
import { installJobErrorKey } from "@/lib/extension-install-job-errors";
import { isTerminalInstall } from "@/lib/extension-install-job-parser";
import { ExtensionInstallJobRow, type ExtensionInstallActions } from "./extension-install-job-row";

export function ExtensionInstallRows({ onOpen }: { onOpen: (id: string) => void }) {
  const { t } = useTranslation();
  const tracker = useExtensionInstallJobs();
  const root = useRef<HTMLDivElement>(null);
  // Pending decisions and active work stay reachable above the bounded result history.
  const visibleJobs = [
    ...tracker.jobs.filter(job => !isTerminalInstall(job.status)),
    ...tracker.jobs.filter(job => isTerminalInstall(job.status)).reverse(),
  ];
  const actions: ExtensionInstallActions = {
    cancel: id => { void tracker.action("cancel_extension_install", id); },
    continue: (id, nonce) => { void tracker.action("continue_extension_install", id, nonce); },
    resume: id => { void tracker.action("resume_extension_install", id); },
    retry: id => { void tracker.action("retry_extension_install", id); },
    dismiss: id => { void tracker.action("dismiss_extension_install", id); },
    open: onOpen,
  };
  function showRequest(id: string) {
    const row = Array.from(root.current?.querySelectorAll<HTMLElement>("[data-install-job]") ?? [])
      .find(element => element.dataset.installJob === id);
    row?.focus();
    row?.scrollIntoView({ block: "nearest" });
  }
  return <div ref={root} className="eij-list">
    {tracker.loadError && <div className="update-bubble eij-root" role="alert">
      <p>{t(tracker.loadError)}</p>
      <button className="btn btn-sm" type="button" onClick={() => void tracker.refresh()}>{t("extensionInstalls.retry")}</button>
    </div>}
    {visibleJobs.map(job => <ExtensionInstallJobRow key={job.id} job={job}
      blockerName={tracker.jobs.find(other => other.id === job.queueBlocker?.jobId)?.displayName}
      busy={tracker.busyIds.has(job.id)}
      errorKey={tracker.errors[job.id] ?? (job.errorCode ? installJobErrorKey(job.errorCode) : null)}
      actions={actions} onShowRequest={showRequest} />)}
  </div>;
}
