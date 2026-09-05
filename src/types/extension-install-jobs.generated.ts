// Generated from Rust install_jobs/types.rs. Do not edit.
export type InstallRequest = { "kind": "local", path: string, } | { "kind": "git", locator: string, } | { "kind": "npm", locator: string, } | { "kind": "update", extensionId: string, };
export type InstallKind = "local" | "git" | "npm" | "update";
export type InstallStatus = "queued" | "running" | "awaitingConfirmation" | "cancelling" | "completed" | "cancelled" | "failed" | "interrupted";
export type InstallPhase = "resolving" | "downloading" | "dependencies" | "validating" | "buildingUi" | "publishing" | "cleaning";
export type QueueBlocker = { "kind": "confirmation", jobId: string, };
export type InstallJobView = { id: string, revision: number, kind: InstallKind, displayName: string, status: InstallStatus, phase: InstallPhase, downloadedBytes: number | null, downloadTotalBytes: number | null, occupiedBytes: number, freeBytes: number | null, confirmationId: string | null, errorCode: string | null, extensionId: string | null, canCancel: boolean, canResume: boolean, queueBlocker: QueueBlocker | null, };
export type InstallJobsSnapshot = { revision: number, jobs: Array<InstallJobView>, };
