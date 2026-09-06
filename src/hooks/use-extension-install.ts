import { useCallback, type Dispatch, type SetStateAction } from "react";
import type { ExtensionInstallOutcome } from "@/lib/extension-install";
import type { InstallRequest } from "@/types/extension-install-jobs.generated";
import { useExtensionInstallJobs } from "./use-extension-install-jobs";

export function useExtensionInstall() {
  const { start } = useExtensionInstallJobs();
  return useCallback(async (request: InstallRequest): Promise<ExtensionInstallOutcome> => {
    const result = await start(request);
    return { jobId: result.job?.id ?? null, errorKey: result.errorKey };
  }, [start]);
}

export function useExtensionUpdate(
  install: ReturnType<typeof useExtensionInstall>,
  setOperationError: Dispatch<SetStateAction<string | null>>,
) {
  return useCallback(async (id: string) => {
    setOperationError(null);
    const result = await install({ kind: "update", extensionId: id });
    setOperationError(result.errorKey);
    return result;
  }, [install, setOperationError]);
}
