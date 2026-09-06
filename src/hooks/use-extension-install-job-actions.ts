import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { INSTALL_JOB_LIMITS, type InstallJobView, type InstallRequest } from "@/types/extension-install-jobs.generated";
import { installJobId, parseInstallJob } from "@/lib/extension-install-job-parser";
import { installJobErrorKey } from "@/lib/extension-install-job-errors";

const MAX_ACTIONS = INSTALL_JOB_LIMITS.active + INSTALL_JOB_LIMITS.recent + 1;
interface ActionResult { job: InstallJobView | null; errorKey: string | null; ok: boolean }
type JobCommand = "cancel_extension_install" | "continue_extension_install" | "resume_extension_install" | "retry_extension_install" | "dismiss_extension_install";

export function useExtensionInstallJobActions(refresh: () => Promise<void>) {
  const pending = useRef(new Set<string>());
  const [busyIds, setBusyIds] = useState<ReadonlySet<string>>(new Set());
  const [errors, setErrors] = useState<Record<string, string>>({});
  const execute = useCallback(async (key: string, command: string, args: Record<string, unknown>): Promise<ActionResult> => {
    if (pending.current.has(key) || pending.current.size >= MAX_ACTIONS) return { job: null, errorKey: "extensionInstalls.errors.busy", ok: false };
    pending.current.add(key);
    setBusyIds(new Set(pending.current));
    setErrors(current => Object.fromEntries(Object.entries(current).filter(([id]) => id !== key)));
    try {
      const value = await invoke<unknown>(command, args);
      const result = command === "dismiss_extension_install" ? null : parseInstallJob(value);
      // Admission is complete already; listing must not keep the add dialog open.
      void refresh();
      return { job: result, errorKey: null, ok: true };
    } catch (error) {
      const errorKey = installJobErrorKey(error);
      setErrors(current => Object.fromEntries([
        ...Object.entries(current).filter(([id]) => id !== key).slice(-(MAX_ACTIONS - 1)),
        [key, errorKey],
      ]));
      return { job: null, errorKey, ok: false };
    } finally {
      pending.current.delete(key);
      setBusyIds(new Set(pending.current));
    }
  }, [refresh]);
  const start = useCallback((request: InstallRequest) =>
    execute("start", "start_extension_install", { request }), [execute]);
  const action = useCallback(async (command: JobCommand, id: string, confirmationId?: string) => {
    installJobId(id);
    if (confirmationId !== undefined) installJobId(confirmationId);
    return execute(id, command, { jobId: id, ...(confirmationId ? { confirmationId } : {}) });
  }, [execute]);
  return { start, action, busyIds, errors };
}
