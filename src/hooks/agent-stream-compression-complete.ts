import { invoke } from "@tauri-apps/api/core";
import type { StreamRecord } from "./agent-stream-cleanup";
import { resolveSessionContext } from "./agent-token-estimate";
import type { AgentSession } from "@/types/agent";

export function handleCompressionComplete(
  sessionId: string,
  record: StreamRecord,
  notify: (record: StreamRecord) => void,
  notifyActivity: (sessionId: string, record: StreamRecord) => void,
) {
  invoke<AgentSession>(
    "get_agent_session", { id: sessionId },
  ).then((session) => {
    const context = resolveSessionContext(session);
    record.state = {
      ...record.state,
      messages: session.messages,
      completedSegments: [],
      currentContent: "",
      currentThinking: "",
      currentTools: [],
      activeStreamItem: null,
      liveTokenCount: 0,
      streamStartedAt: null,
      segmentStartedAt: null,
      isStreaming: false,
      isCompressing: false,
      ...context,
      contextInputTokens: context.sessionTokenCount,
      contextOutputTokens: 0,
      contextLimitTokens: 0,
      persisted: true,
    };
    notify(record);
    notifyActivity(sessionId, record);
  }).catch(() => {});
}
