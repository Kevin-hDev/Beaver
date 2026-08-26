import { useRef, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { agentStreamManager, type StreamSnapshot } from "./agent-stream-manager";
import type { AgentMessage } from "@/types/agent";
import type {
  ChatStreamAdmission,
  NewUserTurnInput,
  TurnStart,
} from "@/types/agent-turn.generated";
import type { StreamKind } from "./agent-chat-stream-types";
import i18n from "@/i18n";
import { admissionErrorMessage } from "@/lib/admission-error";
import { showToast } from "@/lib/toast-emitter";
import { awaitPendingReasoning } from "./session-reasoning-mutation";
import type { StreamRun } from "./agent-stream-run-ownership";
import {
  MAX_PENDING_ADMISSION,
  takePendingForSession,
  type PendingAdmission,
} from "./agent-stream-pending-intents";

interface StreamStartState {
  displayMessages: AgentMessage[];
  baseTokenCount: number;
}

export type StopStreamResult = "ignored" | "stopping" | "stopped";

function resolveStreamKind(turn: TurnStart): StreamKind {
  return turn.type === "new" && turn.input.content.trim() === "/compress"
    ? "compression"
    : "chat";
}

function removePending(items: PendingAdmission[], sessionId: string, runId?: number) {
  for (const item of takePendingForSession(items, sessionId, runId)) {
    agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
  }
}

export function useAgentStream() {
  const ownerRef = useRef(Symbol("agent-stream-owner"));
  const runRef = useRef(0);
  const pendingAdmissionRef = useRef<PendingAdmission[]>([]);

  useEffect(() => () => {
    agentStreamManager.releaseOwner(ownerRef.current);
    for (const item of pendingAdmissionRef.current.splice(0)) {
      agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
    }
  }, []);

  const startStream = useCallback(async (
    sessionId: string,
    model: string,
    provider: string,
    turn: TurnStart,
    _think: boolean,
    startState: StreamStartState,
    workingDir?: string,
    _supportsTools?: boolean,
    _supportsThinking?: boolean,
    _supportsVision?: boolean,
    _reasoningMode?: string | null,
    permissionMode?: string,
    planMode?: boolean,
    optimisticUserMessageId?: string,
  ) => {
    removePending(pendingAdmissionRef.current, sessionId);
    const run: StreamRun = { owner: ownerRef.current, id: ++runRef.current };
    await agentStreamManager.startSession(
      sessionId,
      startState.displayMessages,
      startState.baseTokenCount,
      resolveStreamKind(turn),
      true,
      run,
    );

    try {
      await awaitPendingReasoning(sessionId);
      const admission = await invoke<ChatStreamAdmission>("chat_stream", {
        request: {
          sessionId, model, provider, turn,
          workingDir: workingDir ?? null,
          permissionMode: permissionMode ?? null,
          planMode: planMode ?? null,
        },
      });
      const stopClaim = agentStreamManager.getDeferredStop(
        sessionId, run, admission.generation,
      );
      if (stopClaim) {
        try {
          await invoke("cancel_agent_request", { sessionId, generation: admission.generation });
          if (agentStreamManager.completeStop(sessionId, stopClaim)) {
            removePending(pendingAdmissionRef.current, sessionId, stopClaim.runId);
          }
          return;
        } catch {
          if (!agentStreamManager.releaseStop(sessionId, stopClaim)) return;
        }
      }
      if (!agentStreamManager.ownsRun(sessionId, run)) {
        await invoke("cancel_agent_request", {
          sessionId,
          generation: admission.generation,
        }).catch(() => {});
        return;
      }
      agentStreamManager.reconcileTurnAdmission(
        sessionId,
        admission,
        optimisticUserMessageId,
      );
      const adoption = agentStreamManager.setSessionGeneration(sessionId, admission.generation);
      if (adoption !== "accepted") {
        if (adoption === "rejected") agentStreamManager.failSession(sessionId);
        removePending(pendingAdmissionRef.current, sessionId, run.id);
        await invoke("cancel_agent_request", {
          sessionId,
          generation: admission.generation,
        }).catch(() => {});
        return;
      }
      const pending = takePendingForSession(pendingAdmissionRef.current, sessionId);
      for (const item of pending) {
        try {
          const queued = await invoke<boolean>("queue_agent_message", {
            sessionId: item.sessionId, generation: admission.generation, input: item.input,
          });
          if (queued) continue;
        } catch (error) {
          showToast(admissionErrorMessage(error, i18n.t), "error");
        }
        agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
      }
    } catch (error) {
      if (!agentStreamManager.matchesRun(sessionId, run)) return;
      agentStreamManager.failSession(
        sessionId,
        admissionErrorMessage(error, i18n.t, "errors.streamStartFailed"),
      );
      removePending(pendingAdmissionRef.current, sessionId, run.id);
    }
  }, []);

  const queueStreamMessage = useCallback(async (
    sessionId: string,
    input: NewUserTurnInput,
    displayMessage: AgentMessage,
  ): Promise<boolean> => {
    if (!agentStreamManager.ownsOwner(sessionId, ownerRef.current)
        && !agentStreamManager.adoptOwner(sessionId, ownerRef.current)) return false;
    const runId = agentStreamManager.getOwnedRunId(sessionId, ownerRef.current);
    if (runId === null) return false;
    const generation = agentStreamManager.getOwnedGeneration(sessionId, ownerRef.current);
    if (generation === null) {
      if (pendingAdmissionRef.current.length >= MAX_PENDING_ADMISSION
        || !agentStreamManager.queueUserMessage(sessionId, displayMessage)) {
        return false;
      }
      pendingAdmissionRef.current.push({ sessionId, runId, input, displayMessage });
      return true;
    }
    if (!agentStreamManager.queueUserMessage(sessionId, displayMessage)) {
      return false;
    }
    try {
      const queued = await invoke<boolean>("queue_agent_message", {
        sessionId,
        generation,
        input,
      });
      if (queued) return true;
    } catch (error) {
      showToast(admissionErrorMessage(error, i18n.t), "error");
    }
    agentStreamManager.removeQueuedUserMessage(sessionId, displayMessage.id);
    return false;
  }, []);

  const stopStream = useCallback(async (sessionId: string): Promise<StopStreamResult> => {
    const claim = agentStreamManager.claimStop(sessionId, ownerRef.current);
    if (claim === null) return "ignored";
    if (claim.kind === "pending") return "stopping";
    const { generation } = claim;
    try {
      await invoke("cancel_agent_request", { sessionId, generation });
    } catch {
      agentStreamManager.releaseStop(sessionId, claim);
      return "ignored";
    }
    const stopped = agentStreamManager.completeStop(sessionId, claim);
    if (!stopped) return "ignored";
    removePending(pendingAdmissionRef.current, sessionId, claim.runId);
    return "stopped";
  }, []);

  const subscribeToStream = useCallback(
    (sessionId: string, listener: (snapshot: StreamSnapshot) => void) => {
      agentStreamManager.adoptOwner(sessionId, ownerRef.current);
      return agentStreamManager.subscribe(sessionId, listener);
    },
    [],
  );

  const getStreamSnapshot = useCallback(
    (sessionId: string) => agentStreamManager.getSnapshot(sessionId),
    [],
  );

  return {
    startStream,
    queueStreamMessage,
    stopStream,
    subscribeToStream,
    getStreamSnapshot,
    isStreaming: (sessionId?: string) =>
      sessionId
        ? agentStreamManager.isStreaming(sessionId)
        : agentStreamManager.isOwnerStreaming(ownerRef.current),
  };
}
