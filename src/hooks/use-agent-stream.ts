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

interface StreamStartState {
  displayMessages: AgentMessage[];
  baseTokenCount: number;
}

interface PendingAdmission {
  sessionId: string;
  input: NewUserTurnInput;
  displayMessage: AgentMessage;
}

const MAX_PENDING_ADMISSION = 8;

function resolveStreamKind(turn: TurnStart): StreamKind {
  return turn.type === "new" && turn.input.content.trim() === "/compress"
    ? "compression"
    : "chat";
}

function takePendingForSession(items: PendingAdmission[], sessionId: string) {
  const selected = items.filter((item) => item.sessionId === sessionId);
  const retained = items.filter((item) => item.sessionId !== sessionId);
  items.splice(0, items.length, ...retained);
  return selected;
}

export function useAgentStream() {
  const ownerRef = useRef(Symbol("agent-stream-owner"));
  const runRef = useRef(0);
  const pendingAdmissionRef = useRef<PendingAdmission[]>([]);

  useEffect(() => () => {
    agentStreamManager.discardOwner(ownerRef.current);
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
    for (const item of takePendingForSession(pendingAdmissionRef.current, sessionId)) {
      agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
    }
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
        for (const item of takePendingForSession(pendingAdmissionRef.current, sessionId)) {
          agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
        }
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
      if (!agentStreamManager.ownsRun(sessionId, run)) return;
      agentStreamManager.failSession(
        sessionId,
        admissionErrorMessage(error, i18n.t, "errors.streamStartFailed"),
      );
      for (const item of takePendingForSession(pendingAdmissionRef.current, sessionId)) {
        agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
      }
    }
  }, []);

  const queueStreamMessage = useCallback(async (
    sessionId: string,
    input: NewUserTurnInput,
    displayMessage: AgentMessage,
  ): Promise<boolean> => {
    if (!agentStreamManager.ownsOwner(sessionId, ownerRef.current)) return false;
    const generation = agentStreamManager.getOwnedGeneration(sessionId, ownerRef.current);
    if (generation === null) {
      if (pendingAdmissionRef.current.length >= MAX_PENDING_ADMISSION
        || !agentStreamManager.queueUserMessage(sessionId, displayMessage)) {
        return false;
      }
      pendingAdmissionRef.current.push({ sessionId, input, displayMessage });
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

  const stopStream = useCallback(async (sessionId: string): Promise<boolean> => {
    const generation = agentStreamManager.claimStop(sessionId, ownerRef.current);
    if (generation === null) return false;
    try {
      await invoke("cancel_agent_request", { sessionId, generation });
    } catch {
      agentStreamManager.releaseStop(sessionId, ownerRef.current, generation);
      return false;
    }
    const stopped = agentStreamManager.completeStop(sessionId, ownerRef.current, generation);
    if (!stopped) return false;
    for (const item of takePendingForSession(pendingAdmissionRef.current, sessionId)) {
      agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
    }
    return true;
  }, []);

  const subscribeToStream = useCallback(
    (sessionId: string, listener: (snapshot: StreamSnapshot) => void) =>
      agentStreamManager.subscribe(sessionId, listener),
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
