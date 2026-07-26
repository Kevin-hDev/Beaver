import { invoke } from "@tauri-apps/api/core";
import type { StreamRecord } from "./agent-stream-cleanup";
import { estimateAgentMessagesTokens } from "./agent-token-estimate";
import type { AgentMessage } from "@/types/agent";

export function handleCompressionComplete(
  sessionId: string,
  record: StreamRecord,
  notify: (record: StreamRecord) => void,
  notifyActivity: (sessionId: string, record: StreamRecord) => void,
) {
  invoke<{ messages: AgentMessage[]; accumulated_tokens: number }>(
    "get_agent_session", { id: sessionId },
  ).then((session) => {
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
      sessionTokenCount: estimateAgentMessagesTokens(session.messages),
      sessionTokenCountEstimated: true,
      persisted: true,
    };
    notify(record);
    notifyActivity(sessionId, record);
  }).catch(() => {});
}
