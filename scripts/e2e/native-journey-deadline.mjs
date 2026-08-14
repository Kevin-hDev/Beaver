import { randomUUID } from "node:crypto";
import { performance } from "node:perf_hooks";

const MAX_TIMEOUT_MS = 10 * 60 * 1_000;
const MAX_STAGE_COUNT = 32;
const STAGE_PATTERN = /^[a-z][a-z0-9_]{0,47}$/u;
const TRACE_PREFIX = "[native-cef-journey] ";

export const NATIVE_JOURNEY_TIMEOUT_MS = 60_000;
export const NATIVE_JOURNEY_CLEANUP_TIMEOUT_MS = 5_000;
const MOCHA_REPORTING_GRACE_MS = 1_000;
export const NATIVE_JOURNEY_MOCHA_TIMEOUT_MS = NATIVE_JOURNEY_TIMEOUT_MS
  + NATIVE_JOURNEY_CLEANUP_TIMEOUT_MS
  + MOCHA_REPORTING_GRACE_MS;

export const NATIVE_CEF_STAGE_CEILINGS_MS = Object.freeze({
  page_server_start: 5_000,
  onboarding: 15_000,
  native_webviews: 10_000,
  browser_capability: 15_000,
  browser_session_open: 5_000,
  browser_surface: 10_000,
  cef_helper_start: 15_000,
  cef_helper_turnover: 20_000,
  page_load: 15_000,
  browser_capability_after_turnover: 5_000,
  browser_session_after_turnover: 5_000,
  exit_request: 5_000,
  webdriver_release: 10_000,
  owned_process_exit: 20_000,
  native_webview_exit: 20_000,
});

class NativeJourneyStageError extends Error {
  constructor(stage, code, cause) {
    super(`Native journey stage failed: ${stage}`, cause ? { cause } : undefined);
    this.name = "NativeJourneyStageError";
    this.stage = stage;
    this.code = code;
  }
}

export function createNativeJourney({
  timeoutMs = NATIVE_JOURNEY_TIMEOUT_MS,
  cleanupTimeoutMs = NATIVE_JOURNEY_CLEANUP_TIMEOUT_MS,
  stageCeilings = NATIVE_CEF_STAGE_CEILINGS_MS,
  now = () => performance.now(),
  report = reportToStdout,
} = {}) {
  validateTimeout(timeoutMs);
  validateTimeout(cleanupTimeoutMs);
  validateStageCeilings(stageCeilings);
  if (typeof now !== "function" || typeof report !== "function") {
    throw new Error("Native journey configuration is invalid");
  }
  const journeyId = randomUUID();
  const ceilings = Object.freeze({ ...stageCeilings });
  const startedAt = readNow(now);
  const deadline = startedAt + timeoutMs;
  let cleanupDeadline;

  return {
    async run(stage, operation) {
      validateKnownStage(stage, ceilings);
      return runBounded({
        journeyId,
        stage,
        operation,
        deadline,
        ceilingMs: ceilings[stage],
        now,
        report,
      });
    },
    async cleanup(stage, operation) {
      validateStage(stage);
      cleanupDeadline ??= readNow(now) + cleanupTimeoutMs;
      return runBounded({
        journeyId,
        stage,
        operation,
        deadline: cleanupDeadline,
        ceilingMs: cleanupTimeoutMs,
        now,
        report,
      });
    },
  };
}

async function runBounded({
  journeyId, stage, operation, deadline, ceilingMs, now, report,
}) {
  if (typeof operation !== "function") {
    throw new Error("Native journey operation is invalid");
  }
  const stageStartedAt = readNow(now);
  const remainingMs = Math.floor(deadline - stageStartedAt);
  if (remainingMs <= 0) {
    reportEvent(report, journeyId, stage, "failed", 0, "journey-timeout");
    throw new NativeJourneyStageError(stage, "journey-timeout");
  }
  const timeoutMs = Math.max(1, Math.min(ceilingMs, remainingMs));
  const controller = new AbortController();
  const timeoutMarker = new Error("Native journey timeout marker");
  let timer;
  reportEvent(report, journeyId, stage, "started", 0);
  try {
    const timeout = new Promise((_, reject) => {
      timer = setTimeout(() => {
        controller.abort();
        reject(timeoutMarker);
      }, timeoutMs);
    });
    const result = await Promise.race([
      Promise.resolve().then(() => operation({ signal: controller.signal, timeoutMs })),
      timeout,
    ]);
    reportEvent(report, journeyId, stage, "completed", elapsed(now, stageStartedAt));
    return result;
  } catch (error) {
    const timedOut = error === timeoutMarker;
    const code = timedOut ? "stage-timeout" : "stage-error";
    reportEvent(report, journeyId, stage, "failed", elapsed(now, stageStartedAt), code);
    throw new NativeJourneyStageError(stage, code, timedOut ? undefined : error);
  } finally {
    clearTimeout(timer);
  }
}

function validateStageCeilings(stageCeilings) {
  if (!stageCeilings || typeof stageCeilings !== "object" || Array.isArray(stageCeilings)) {
    throw new Error("Native journey configuration is invalid");
  }
  const entries = Object.entries(stageCeilings);
  if (entries.length === 0 || entries.length > MAX_STAGE_COUNT) {
    throw new Error("Native journey configuration is invalid");
  }
  for (const [stage, timeoutMs] of entries) {
    validateStage(stage);
    validateTimeout(timeoutMs);
  }
}

function validateKnownStage(stage, stageCeilings) {
  validateStage(stage);
  if (!Object.hasOwn(stageCeilings, stage)) {
    throw new Error("Native journey stage is invalid");
  }
}

function validateStage(stage) {
  if (typeof stage !== "string" || !STAGE_PATTERN.test(stage)) {
    throw new Error("Native journey stage is invalid");
  }
}

function validateTimeout(timeoutMs) {
  if (!Number.isSafeInteger(timeoutMs) || timeoutMs < 1 || timeoutMs > MAX_TIMEOUT_MS) {
    throw new Error("Native journey timeout is invalid");
  }
}

function readNow(now) {
  const value = now();
  if (!Number.isFinite(value) || value < 0) {
    throw new Error("Native journey clock is invalid");
  }
  return value;
}

function elapsed(now, startedAt) {
  return Math.max(0, Math.min(MAX_TIMEOUT_MS, Math.round(readNow(now) - startedAt)));
}

function reportEvent(report, journeyId, stage, state, elapsedMs, code) {
  report({ journeyId, stage, state, elapsedMs, ...(code ? { code } : {}) });
}

function reportToStdout(event) {
  process.stdout.write(`${TRACE_PREFIX}${JSON.stringify(event)}\n`);
}
