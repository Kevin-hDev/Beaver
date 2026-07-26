import { estimateAgentMessagesTokens } from "./agent-token-estimate";
import { markSubagentSnapshot } from "./agent-stream-persistence-owner";
import type { StreamRecord } from "./agent-stream-cleanup";
import type { AgentMessage } from "@/types/agent";

export function applySessionSnapshot(record: StreamRecord, messages: AgentMessage[]) {
  markSubagentSnapshot(record);
  record.state = {
    ...record.state,
    messages,
    streamRunId: latestOpenStreamRunId(messages) ?? record.state.streamRunId,
    sessionTokenCount: estimateAgentMessagesTokens(messages),
    sessionTokenCountEstimated: true,
    isStreaming: true,
    activeStreamItem: null,
    persisted: false,
    completed: false,
  };
}

function latestOpenStreamRunId(messages: AgentMessage[]): string | null {
  const completed = new Set<string>();
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (!message.stream_run_id) continue;
    if (message.stream_part === "final") {
      completed.add(message.stream_run_id);
      continue;
    }
    if (!completed.has(message.stream_run_id)) return message.stream_run_id;
  }
  return null;
}
