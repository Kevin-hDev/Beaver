import type { AgentMessage } from "@/types/agent";

const MAX_STREAM_GROUPS = 64;
const MAX_MESSAGES_PER_GROUP = 256;
const RUN_ID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export interface StreamEndArtifactGroup {
  ownerId: string;
  messages: AgentMessage[];
}

export function planStreamEndArtifacts(
  messages: AgentMessage[],
  isStreaming: boolean,
  activeRunId: string,
): Map<string, StreamEndArtifactGroup> {
  const placements = new Map<string, StreamEndArtifactGroup>();
  const groups = collectGroups(messages);

  for (const message of messages) {
    if (message.role !== "assistant" || validRunId(message.stream_run_id)) continue;
    placements.set(message.id, { ownerId: message.id, messages: [message] });
  }

  for (const [runId, grouped] of groups) {
    if (isStreaming && runId === activeRunId) continue;
    const final = findLast(grouped, (message) => (
      message.role === "assistant" && message.stream_part === "final"
    ));
    const owner = final ?? grouped[grouped.length - 1];
    const assistantMessages = grouped.filter((message) => message.role === "assistant");
    if (owner && assistantMessages.length > 0) {
      placements.set(owner.id, { ownerId: owner.id, messages: assistantMessages });
    }
  }
  return placements;
}

function collectGroups(messages: AgentMessage[]): Map<string, AgentMessage[]> {
  const groups = new Map<string, AgentMessage[]>();
  for (const message of messages) {
    const runId = message.stream_run_id;
    if (!validRunId(runId)) continue;
    let group = groups.get(runId);
    if (!group) {
      if (groups.size >= MAX_STREAM_GROUPS) continue;
      group = [];
      groups.set(runId, group);
    }
    if (group.length < MAX_MESSAGES_PER_GROUP) group.push(message);
  }
  return groups;
}

function validRunId(value?: string): value is string {
  return typeof value === "string" && RUN_ID_RE.test(value);
}

function findLast<T>(items: T[], predicate: (item: T) => boolean): T | undefined {
  for (let index = items.length - 1; index >= 0; index -= 1) {
    if (predicate(items[index])) return items[index];
  }
  return undefined;
}
