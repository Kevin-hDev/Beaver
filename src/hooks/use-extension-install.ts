import { useCallback, type Dispatch, type SetStateAction } from "react";
import type { ExtensionInstallOutcome } from "@/lib/extension-install";
import type { InstallRequest } from "@/types/extension-install-jobs.generated";
import { useExtensionInstallJobs } from "./use-extension-install-jobs";

export function useExtensionInstall(setOperationError: Dispatch<SetStateAction<string | null>>) {
  const { start } = useExtensionInstallJobs();
  return useCallback(async (request: InstallRequest): Promise<ExtensionInstallOutcome> => {
    setOperationError(null);
    const result = await start(request);
    setOperationError(result.errorKey);
    return { jobId: result.job?.id ?? null, errorKey: result.errorKey };
  }, [start, setOperationError]);
}
