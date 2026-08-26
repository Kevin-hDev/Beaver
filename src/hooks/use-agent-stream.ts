import { useRef, useCallback } from "react";
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

interface StreamStartState {
  displayMessages: AgentMessage[];
  baseTokenCount: number;
}

function resolveStreamKind(turn: TurnStart): StreamKind {
  return turn.type === "new" && turn.input.content.trim() === "/compress"
    ? "compression"
    : "chat";
}

export function useAgentStream() {
  const streamingRef = useRef(false);
  const generationRef = useRef<number | null>(null);
  const runRef = useRef(0);
  const stoppingRef = useRef(false);

  const startStream = useCallback(async (
    sessionId: string,
    model: string,
    provider: string,
    turn: TurnStart,
    think: boolean,
    startState: StreamStartState,
    workingDir?: string,
    supportsTools?: boolean,
    supportsThinking?: boolean,
    supportsVision?: boolean,
    reasoningMode?: string | null,
    permissionMode?: string,
    planMode?: boolean,
    optimisticUserMessageId?: string,
  ) => {
    const run = ++runRef.current;
    stoppingRef.current = false;
    streamingRef.current = true;
    await agentStreamManager.startSession(
      sessionId,
      startState.displayMessages,
      startState.baseTokenCount,
      resolveStreamKind(turn),
    );

    try {
      const admission = await invoke<ChatStreamAdmission>("chat_stream", {
        sessionId,
        model,
        provider,
        turn,
        tools: [],
        think,
        workingDir: workingDir ?? null,
        supportsTools: supportsTools ?? null,
        supportsThinking: supportsThinking ?? null,
        supportsVision: supportsVision ?? null,
        reasoningMode: reasoningMode ?? null,
        permissionMode: permissionMode ?? null,
        planMode: planMode ?? null,
      });
      if (runRef.current !== run || stoppingRef.current) {
        agentStreamManager.stopSession(sessionId, admission.generation);
        await invoke("cancel_agent_request", {
          sessionId,
          generation: admission.generation,
        }).catch(() => {});
        return;
      }
      generationRef.current = admission.generation;
      agentStreamManager.setSessionGeneration(sessionId, admission.generation);
      agentStreamManager.reconcileTurnAdmission(
        sessionId,
        admission,
        optimisticUserMessageId,
      );
    } catch (error) {
      agentStreamManager.failSession(
        sessionId,
        admissionErrorMessage(error, i18n.t, "errors.streamStartFailed"),
      );
      streamingRef.current = false;
    }
  }, []);

  const queueStreamMessage = useCallback(async (
    sessionId: string,
    input: NewUserTurnInput,
    displayMessage: AgentMessage,
  ): Promise<boolean> => {
    const generation = generationRef.current;
    if (generation === null || !agentStreamManager.queueUserMessage(sessionId, displayMessage)) {
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
