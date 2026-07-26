import { useCallback, useEffect, useRef, useState } from "react";
import {
  finishCloneSummaryRun,
  getCloneSummaryRun,
  setCloneSummaryRunVisible,
  startCloneSummaryRun,
  useCloneSummaryRun,
} from "./clone-summary-runs";
import type { AgentMessage, CloneMode } from "@/types/agent";

export interface PendingCloneDialog {
  messageId: string;
  canSummarize: boolean;
  error?: string | null;
}

interface CloneActivationOptions {
  operationId?: string;
  shouldActivateOnComplete?: () => boolean;
}

export type CloneMessageHandler = (
  messageId: string,
  mode: CloneMode,
  customFocus?: string,
  options?: CloneActivationOptions,
) => Promise<void>;

export function useChatClone(
  sessionId: string,
  messages: AgentMessage[],
  onCloneMessage?: CloneMessageHandler,
  onCancelCloneSummary?: (operationId: string) => Promise<void>,
) {
  const [pendingClone, setPendingClone] = useState<PendingCloneDialog | null>(null);
  const [cloneBusy, setCloneBusy] = useState(false);
  const summaryRun = useCloneSummaryRun(sessionId);
  const pendingCloneRef = useRef<PendingCloneDialog | null>(null);
  const runIdRef = useRef(0);
  const operationInFlightRef = useRef(false);

  useEffect(() => {
    pendingCloneRef.current = pendingClone;
  }, [pendingClone]);

  const requestClone = useCallback((id: string) => {
    if (operationInFlightRef.current) return;
    const index = messages.findIndex((message) => message.id === id);
    const next = {
      messageId: id,
      canSummarize: index >= 0 && index < messages.length - 1,
      error: null,
    };
    pendingCloneRef.current = next;
    setPendingClone(next);
  }, [messages]);

  const submitClone = useCallback(async (mode: CloneMode, customFocus?: string) => {
    if (!pendingClone || !onCloneMessage || operationInFlightRef.current) return;
    if (mode === "summary" && getCloneSummaryRun(sessionId)) return;
    operationInFlightRef.current = true;
    const runId = ++runIdRef.current;
    const messageId = pendingClone.messageId;
    const operationId = crypto.randomUUID();
    const isSummary = mode === "summary";
    setCloneBusy(isSummary);
    if (!isSummary) {
      pendingCloneRef.current = null;
      setPendingClone(null);
    }
    if (mode === "summary") {
      startCloneSummaryRun({ sessionId, messageId, operationId, visible: true });
    }
    try {
      await onCloneMessage(messageId, mode, customFocus, {
        operationId: mode === "summary" ? operationId : undefined,
        shouldActivateOnComplete: () =>
          runIdRef.current === runId
          && (!isSummary || pendingCloneRef.current?.messageId === messageId),
      });
      finishCloneSummaryRun(sessionId, operationId);
      pendingCloneRef.current = null;
      setPendingClone(null);
    } catch {
      if (mode !== "summary") {
        const failedClone = { ...pendingClone, error: "failed" };
        pendingCloneRef.current = failedClone;
        setPendingClone(failedClone);
        return;
      }
      const stillRunning = getCloneSummaryRun(sessionId)?.operationId === operationId;
      if (stillRunning) {
        finishCloneSummaryRun(sessionId, operationId);
        setPendingClone((current) => current ? { ...current, error: "failed" } : current);
      }
    } finally {
      operationInFlightRef.current = false;
      if (runIdRef.current === runId) setCloneBusy(false);
    }
  }, [onCloneMessage, pendingClone, sessionId]);

  const cancelClone = useCallback(() => {
    const run = getCloneSummaryRun(sessionId);
    pendingCloneRef.current = null;
    if (run) {
      setPendingClone(null);
      setCloneSummaryRunVisible(sessionId, false);
      return;
    }
    setPendingClone(null);
    setCloneBusy(false);
  }, [sessionId]);

  const showRunningClone = useCallback(() => {
    const run = getCloneSummaryRun(sessionId);
    if (!run) return;
    pendingCloneRef.current = { messageId: run.messageId, canSummarize: true, error: null };
    setCloneSummaryRunVisible(sessionId, true);
  }, [sessionId]);

  const abortClone = useCallback(async () => {
    const run = getCloneSummaryRun(sessionId);
    pendingCloneRef.current = null;
    setPendingClone(null);
    if (!run) {
      setCloneBusy(false);
      return;
    }
    finishCloneSummaryRun(sessionId, run.operationId);
    setCloneBusy(false);
    await onCancelCloneSummary?.(run.operationId);
  }, [onCancelCloneSummary, sessionId]);

  const dialogClone = summaryRun?.visible
    ? { messageId: summaryRun.messageId, canSummarize: true, error: null }
    : pendingClone;

  return {
    pendingClone: dialogClone,
    cloneBusy: cloneBusy || Boolean(summaryRun?.visible),
    summaryRun,
    requestClone,
    submitClone,
    cancelClone,
    showRunningClone,
    abortClone,
  };
}
