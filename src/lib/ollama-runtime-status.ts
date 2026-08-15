import type {
  BundleState,
  DaemonState,
  OllamaProgressStage,
  OllamaRuntimeStatus,
  OperationState,
} from "@/types/ollama-runtime";
import { isOllamaErrorCode } from "@/lib/ollama-runtime-error";

const BUNDLE_STATES: ReadonlySet<BundleState> = new Set([
  "absent", "ready", "transaction_pending", "recovery_required",
]);
const OPERATION_STATES: ReadonlySet<OperationState> = new Set([
  "idle", "installing", "updating", "recovering", "cancelling",
]);
const PROGRESS_STAGES: ReadonlySet<OllamaProgressStage> = new Set([
  "preparing", "downloading", "verifying", "extracting", "validating",
  "committing", "starting", "recovering", "rolling_back", "cleaning",
]);

type RecordValue = Record<string, unknown>;

export function parseOllamaRuntimeStatus(value: unknown): OllamaRuntimeStatus | null {
  if (!isRecord(value) || !isBundleState(value.bundle)) return null;
  if (value.bundle === "absent" && Object.keys(value).length === 1) {
    return { bundle: "absent", daemon: "unavailable", operation: "idle", progress: null, last_error: null };
  }
  if (!isDaemonState(value.daemon)
    || !isOperationState(value.operation)
    || !isProgressStage(value.progress)
    || !(value.last_error === null || isOllamaErrorCode(value.last_error))) {
    return null;
  }
  return value as OllamaRuntimeStatus;
}

function isRecord(value: unknown): value is RecordValue {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isBundleState(value: unknown): value is BundleState {
  return typeof value === "string" && BUNDLE_STATES.has(value as BundleState);
}

function isOperationState(value: unknown): value is OperationState {
  return typeof value === "string" && OPERATION_STATES.has(value as OperationState);
}

function isProgressStage(value: unknown): value is OllamaProgressStage | null {
  return value === null || (typeof value === "string" && PROGRESS_STAGES.has(value as OllamaProgressStage));
}

function isDaemonState(value: unknown): value is DaemonState {
  if (value === "unavailable") return true;
  if (!isRecord(value) || Object.keys(value).length !== 1) return false;
  const kind = Object.keys(value)[0];
  if (kind !== "owned" && kind !== "external") return false;
  const daemon = value[kind];
  if (!isRecord(daemon) || Object.keys(daemon).length !== 1 || !isRecord(daemon.endpoint)) return false;
  const port = daemon.endpoint.port;
  return typeof port === "number" && Number.isInteger(port) && port > 0 && port <= 65_535;
}
