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

export function useAgentStream() {
  const streamingRef = useRef(false);
  const generationRef = useRef<number | null>(null);
  const activeSessionRef = useRef<string | null>(null);
  const runRef = useRef(0);
  const stoppingRef = useRef(false);
  const pendingAdmissionRef = useRef<PendingAdmission[]>([]);

  useEffect(() => () => {
    runRef.current += 1;
    streamingRef.current = false;
    const sessionId = activeSessionRef.current;
    activeSessionRef.current = null;
    if (sessionId) agentStreamManager.discardPendingAdmission(sessionId);
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
    if (activeSessionRef.current !== null && activeSessionRef.current !== sessionId) {
      for (const item of pendingAdmissionRef.current.splice(0)) {
        agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
      }
    }
    activeSessionRef.current = sessionId;
    generationRef.current = null;
    const run = ++runRef.current;
    stoppingRef.current = false;
    streamingRef.current = true;
    await agentStreamManager.startSession(
      sessionId,
      startState.displayMessages,
      startState.baseTokenCount,
      resolveStreamKind(turn),
      true,
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
      if (runRef.current !== run || activeSessionRef.current !== sessionId || stoppingRef.current) {
        agentStreamManager.stopSession(sessionId, admission.generation);
        await invoke("cancel_agent_request", {
          sessionId,
          generation: admission.generation,
        }).catch(() => {});
        return;
      }
      generationRef.current = admission.generation;
      agentStreamManager.reconcileTurnAdmission(
        sessionId,
        admission,
        optimisticUserMessageId,
      );
      agentStreamManager.setSessionGeneration(sessionId, admission.generation);
      const pending = pendingAdmissionRef.current.splice(0);
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
      if (runRef.current !== run || activeSessionRef.current !== sessionId) return;
      agentStreamManager.failSession(
        sessionId,
        admissionErrorMessage(error, i18n.t, "errors.streamStartFailed"),
      );
      streamingRef.current = false;
      activeSessionRef.current = null;
      for (const item of pendingAdmissionRef.current.splice(0)) {
        agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
      }
    }
  }, []);

  const queueStreamMessage = useCallback(async (
    sessionId: string,
    input: NewUserTurnInput,
    displayMessage: AgentMessage,
  ): Promise<boolean> => {
    if (activeSessionRef.current !== sessionId) return false;
    const generation = generationRef.current;
    if (generation === null) {
      if (!streamingRef.current
        || pendingAdmissionRef.current.length >= MAX_PENDING_ADMISSION
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

  const stopStream = useCallback(async (sessionId: string) => {
    if (stoppingRef.current) return;
    stoppingRef.current = true;
    runRef.current += 1;
    const gen = generationRef.current;
    generationRef.current = null;
    streamingRef.current = false;
    if (activeSessionRef.current === sessionId) activeSessionRef.current = null;
    for (const item of pendingAdmissionRef.current.splice(0)) {
      agentStreamManager.removeQueuedUserMessage(item.sessionId, item.displayMessage.id);
    }
    agentStreamManager.stopSession(sessionId, gen);
    await invoke("cancel_agent_request", { sessionId, generation: gen }).catch(() => {});
    stoppingRef.current = false;
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
      sessionId ? agentStreamManager.isStreaming(sessionId) : streamingRef.current,
  };
}
