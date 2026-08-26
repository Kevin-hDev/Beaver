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
import type {
  QueueStreamResult,
  StreamRun,
} from "./agent-stream-run-ownership";

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

export function useAgentStream() {
  const ownerRef = useRef(Symbol("agent-stream-owner"));
  const runRef = useRef(0);

  useEffect(() => () => {
    agentStreamManager.releaseOwner(ownerRef.current);
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
          agentStreamManager.completeStop(sessionId, stopClaim);
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
        await invoke("cancel_agent_request", {
          sessionId,
          generation: admission.generation,
        }).catch(() => {});
        return;
      }
    } catch (error) {
      if (!agentStreamManager.matchesRun(sessionId, run)) return;
      agentStreamManager.failSession(
        sessionId,
        admissionErrorMessage(error, i18n.t, "errors.streamStartFailed"),
      );
    }
  }, []);

  const queueStreamMessage = useCallback(async (
    sessionId: string,
    input: NewUserTurnInput,
    displayMessage: AgentMessage,
  ): Promise<QueueStreamResult> => {
    if (!agentStreamManager.ownsOwner(sessionId, ownerRef.current)
        && !agentStreamManager.adoptOwner(sessionId, ownerRef.current)) return "start-new";
    const runState = agentStreamManager.getOwnedRunState(sessionId, ownerRef.current);
    if (runState.kind === "terminal") return "start-new";
    if (runState.kind === "stopping") return "stopping";
    if (runState.kind === "pendingAdmission") {
      return "unavailable";
    }
    if (!agentStreamManager.queueUserMessage(sessionId, displayMessage)) {
      return "start-new";
    }
    try {
      const queued = await invoke<boolean>("queue_agent_message", {
        sessionId, generation: runState.generation,
        input,
      });
      if (queued) return "queued";
    } catch (error) {
      showToast(admissionErrorMessage(error, i18n.t), "error");
    }
    agentStreamManager.removeQueuedUserMessage(sessionId, displayMessage.id);
    return "unavailable";
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
    isStreaming: (sessionId: string) => agentStreamManager.isStreaming(sessionId),
  };
}
