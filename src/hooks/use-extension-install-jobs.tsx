import { createContext, useContext, type ReactNode } from "react";
import { useExtensionInstallSnapshots } from "./use-extension-install-snapshots";
import { useExtensionInstallJobActions } from "./use-extension-install-job-actions";

function useJobs() {
  const state = useExtensionInstallSnapshots();
  const actions = useExtensionInstallJobActions(state.refresh);
  return { jobs: state.snapshot.jobs, loading: state.loading, loadError: state.loadError, refresh: state.refresh, ...actions };
}
const InstallJobsContext = createContext<ReturnType<typeof useJobs> | null>(null);

export function ExtensionInstallJobsProvider({ children }: { children: ReactNode }) {
  const value = useJobs();
  return <InstallJobsContext.Provider value={value}>{children}</InstallJobsContext.Provider>;
}

export function useExtensionInstallJobs() {
  const context = useContext(InstallJobsContext);
  if (!context) throw new Error("ExtensionInstallJobsProvider is missing");
  return context;
}
