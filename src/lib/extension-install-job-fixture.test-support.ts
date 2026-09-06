import type { InstallJobView, InstallJobsSnapshot } from "@/types/extension-install-jobs.generated";

export function installJobFixture(changes: Partial<InstallJobView> = {}): InstallJobView {
  return {
    id: "10000000-0000-4000-8000-000000000001", revision: 1, kind: "npm", displayName: "Fixture",
    status: "running", phase: "dependencies", downloadedBytes: null, downloadTotalBytes: null,
    occupiedBytes: 2048, freeBytes: 4096, confirmationId: null, errorCode: null, extensionId: null,
    canCancel: true, canResume: false, queueBlocker: null, ...changes,
  };
}
export function installSnapshotFixture(revision = 1, jobs = [installJobFixture({ revision })]): InstallJobsSnapshot {
  return { revision, jobs };
}
