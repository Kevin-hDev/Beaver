import { useCallback, useRef, type RefObject } from "react";
import { replaceSessionMessage } from "./agent-chat-turn-revision";
import type { AgentMessage } from "@/types/agent";
import type { TurnStart } from "@/types/agent-turn.generated";
import type { StreamSnapshot } from "./agent-stream-manager";

interface Params {
  sessionId: string | null;
  messages: AgentMessage[];
  permissionModeRef: RefObject<string | undefined>;
  getStreamSnapshot: (id: string) => StreamSnapshot | null;
  syncTokenCount: () => Promise<number>;
  doStream: (
    turn: TurnStart, messages: AgentMessage[], sessionId: string,
    workingDir?: string, tokenCount?: number, permissionMode?: string,
  ) => Promise<void>;
}

export function useAgentTurnRevision({
  sessionId, messages, permissionModeRef, getStreamSnapshot, syncTokenCount, doStream,
}: Params) {
  const pending = useRef(false);
  const revise = useCallback(async (messageId: string, newContent?: string) => {
    // Lock before the first await: a second click must never truncate a running turn.
    if (!sessionId || pending.current || getStreamSnapshot(sessionId)?.isStreaming) return;
    let index = messages.findIndex((message) => message.id === messageId);
    if (index < 0) return;
    if (newContent !== undefined && messages[index].role !== "user") return;
    while (index >= 0 && messages[index].role !== "user") index -= 1;
    if (index < 0) return;
    const message = messages[index];
    const content = newContent ?? message.content;
    pending.current = true;
    try {
      if (!await replaceSessionMessage(sessionId, message.id, content)) return;
      const tokenCount = await syncTokenCount();
      await doStream(
        { type: "resume", input: { message_id: message.id } },
        [...messages.slice(0, index), { ...message, content }],
        sessionId, undefined, tokenCount, permissionModeRef.current,
      );
    } finally {
      pending.current = false;
    }
  }, [sessionId, messages, getStreamSnapshot, syncTokenCount, doStream, permissionModeRef]);
  const reload = useCallback((messageId: string) => revise(messageId), [revise]);
  const edit = useCallback((messageId: string, content: string) => revise(messageId, content), [revise]);
  return { reload, edit };
}
