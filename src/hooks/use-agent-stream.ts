import { useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { agentStreamManager, type StreamSnapshot } from "./agent-stream-manager";
import { resolveAgentStreamMessages } from "./agent-stream-message-resolver";
import type { AgentMessage } from "@/types/agent";
import type { StreamKind } from "./agent-chat-stream-types";

interface StreamStartState {
  displayMessages: AgentMessage[];
  baseTokenCount: number;
}

function resolveStreamKind(messages: AgentMessage[]): StreamKind {
  const lastMessage = messages[messages.length - 1];
  return lastMessage?.role === "user" && lastMessage.content.trim() === "/compress"
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
    messages: AgentMessage[],
    think: boolean,
    startState: StreamStartState,
    workingDir?: string,
    supportsTools?: boolean,
    supportsThinking?: boolean,
    supportsVision?: boolean,
    reasoningMode?: string | null,
    permissionMode?: string,
    planMode?: boolean,
  ) => {
    const run = ++runRef.current;
    stoppingRef.current = false;
    streamingRef.current = true;
    await agentStreamManager.startSession(
      sessionId,
      startState.displayMessages,
      startState.baseTokenCount,
      resolveStreamKind(messages),
    );

    try {
      const chatMessages = await resolveAgentStreamMessages(messages);
      const gen = await invoke<number>("chat_stream", {
        sessionId,
        model,
        provider,
        messages: chatMessages,
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
        agentStreamManager.stopSession(sessionId, gen);
        await invoke("cancel_agent_request", { sessionId, generation: gen }).catch(() => {});
        return;
      }
      generationRef.current = gen;
      agentStreamManager.setSessionGeneration(sessionId, gen);
    } catch {
      agentStreamManager.failSession(sessionId);
      streamingRef.current = false;
    }
  }, []);

  const queueStreamMessage = useCallback(async (
    sessionId: string,
    messages: AgentMessage[],
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
        messages: await resolveAgentStreamMessages(messages),
      });
      if (queued) return true;
    } catch {
      // The generic user feedback is handled by the caller.
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
