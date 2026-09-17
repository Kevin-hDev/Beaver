import { useCallback, type RefObject } from "react";
import { persistAgentMessage, type AgentSendPayload } from "./agent-message-send";
import type { AgentMessage } from "@/types/agent";

interface Params {
  sessionId: string | null;
  messages: AgentMessage[];
  permissionModeRef: RefObject<string | undefined>;
  runOrDefer: (
    workingDir: string | undefined,
    run: (resolvedWorkingDir?: string) => Promise<void>,
  ) => Promise<void>;
  doStream: Parameters<typeof persistAgentMessage>[0]["doStream"];
  queueStreamMessage: Parameters<typeof persistAgentMessage>[0]["queueStreamMessage"];
}

export function useAgentMessageSend(params: Params) {
  const {
    sessionId, messages, permissionModeRef, runOrDefer, doStream, queueStreamMessage,
  } = params;
  const persist = useCallback(async (payload: AgentSendPayload) => {
    if (!sessionId) return false;
    return persistAgentMessage({
      ...payload,
      sessionId,
      messages,
      permissionMode: permissionModeRef.current,
      doStream,
      queueStreamMessage,
    });
  }, [doStream, messages, permissionModeRef, queueStreamMessage, sessionId]);

  return useCallback(async (
    text: string,
    sentFiles?: AgentSendPayload["sentFiles"],
    workingDir?: string,
    projectId?: string,
    skills?: AgentSendPayload["skills"],
  ) => {
    if (!text.trim() && !sentFiles?.length && !skills?.length) return false;
    const payload = { text, sentFiles, workingDir, projectId, skills };
    let accepted = true;
    await runOrDefer(workingDir, async (resolvedWorkingDir) => {
      accepted = await persist({
        ...payload,
        workingDir: resolvedWorkingDir ?? workingDir,
      });
    });
    return accepted;
  }, [persist, runOrDefer]);
}
