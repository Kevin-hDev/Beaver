import { INSTALL_JOB_LIMITS, type InstallJobView, type InstallJobsSnapshot,
  type InstallKind, type InstallStatus, type InstallPhase } from "@/types/extension-install-jobs.generated";
import { LIMITS } from "@/types/extension-contract.generated";
import { identifier, invalid, objectWithKeys, oneOf, text } from "./extension-record-validation";

const KINDS: readonly InstallKind[] = ["local", "git", "npm", "update"];
const STATUSES: readonly InstallStatus[] = ["queued", "running", "awaitingConfirmation", "cancelling", "completed", "cancelled", "failed", "interrupted"];
const PHASES: readonly InstallPhase[] = ["resolving", "downloading", "dependencies", "validating", "buildingUi", "publishing", "cleaning"];
const UUID = /^[\da-f]{8}-[\da-f]{4}-[\da-f]{4}-[\da-f]{4}-[\da-f]{12}$/iu;
const ERROR_CODE = /^[a-z][a-z-]{0,95}$/u;
const VIEW_KEYS: readonly (keyof InstallJobView)[] = ["id", "revision", "kind", "displayName", "status", "phase",
  "downloadedBytes", "downloadTotalBytes", "occupiedBytes", "freeBytes", "confirmationId", "errorCode",
  "extensionId", "canCancel", "canResume", "queueBlocker"];

export function installJobId(value: unknown): string {
  if (typeof value !== "string" || !UUID.test(value)) invalid();
  return value;
}
function integer(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) invalid();
  return value;
}
function boolean(value: unknown): boolean {
  if (typeof value !== "boolean") invalid();
  return value;
}
function nullable<T>(value: unknown, parse: (input: unknown) => T): T | null {
  return value === null ? null : parse(value);
}
export function isTerminalInstall(status: InstallStatus): boolean {
  return status === "completed" || status === "failed" || status === "cancelled" || status === "interrupted";
}
export function parseInstallJob(value: unknown): InstallJobView {
  const input = objectWithKeys(value, VIEW_KEYS);
  const displayName = text(input.displayName, LIMITS.maxExtensionNameChars);
  if (Array.from(displayName).some(char => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127)) invalid();
  const blocker = input.queueBlocker === null ? null : objectWithKeys(input.queueBlocker, ["kind", "jobId"]);
  if (blocker && blocker.kind !== "confirmation") invalid();
  const job: InstallJobView = {
    id: installJobId(input.id), revision: integer(input.revision), kind: oneOf(input.kind, KINDS), displayName,
    status: oneOf(input.status, STATUSES), phase: oneOf(input.phase, PHASES),
    downloadedBytes: nullable(input.downloadedBytes, integer), downloadTotalBytes: nullable(input.downloadTotalBytes, integer),
    occupiedBytes: integer(input.occupiedBytes), freeBytes: nullable(input.freeBytes, integer),
    confirmationId: nullable(input.confirmationId, installJobId), extensionId: nullable(input.extensionId, identifier),
    errorCode: nullable(input.errorCode, (code) => {
      if (typeof code !== "string" || !ERROR_CODE.test(code)) invalid();
      return code;
    }),
    canCancel: boolean(input.canCancel), canResume: boolean(input.canResume),
    queueBlocker: blocker ? { kind: "confirmation", jobId: installJobId(blocker.jobId) } : null,
  };
  if ((job.downloadedBytes !== null && job.downloadTotalBytes !== null && job.downloadedBytes > job.downloadTotalBytes)
    || (job.canResume && job.status !== "interrupted")
    || (job.canCancel && (isTerminalInstall(job.status) || job.status === "cancelling"))
    || (job.confirmationId !== null && job.status !== "awaitingConfirmation")
    || (job.queueBlocker && (job.status !== "queued" || job.queueBlocker.jobId === job.id))) invalid();
  return job;
}
export function parseInstallJobsSnapshot(value: unknown): InstallJobsSnapshot {
  const input = objectWithKeys(value, ["revision", "jobs"]);
  const revision = integer(input.revision);
  if (!Array.isArray(input.jobs) || input.jobs.length > INSTALL_JOB_LIMITS.active + INSTALL_JOB_LIMITS.recent) invalid();
  const jobs = input.jobs.map(parseInstallJob);
  const ids = new Set(jobs.map(job => job.id));
  if (ids.size !== jobs.length || jobs.some(job => job.revision > revision)
    || jobs.filter(job => !isTerminalInstall(job.status)).length > INSTALL_JOB_LIMITS.active
    || jobs.filter(job => isTerminalInstall(job.status)).length > INSTALL_JOB_LIMITS.recent) invalid();
  for (const job of jobs) {
    if (job.queueBlocker && !jobs.some(other => other.id === job.queueBlocker?.jobId && other.status === "awaitingConfirmation")) invalid();
  }
  return { revision, jobs };
}
