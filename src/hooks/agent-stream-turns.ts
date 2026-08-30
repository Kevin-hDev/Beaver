import { flushFrameNotify } from "./agent-stream-notify";
import { notifyRecord, notifyRecordActivity } from "./agent-stream-notify-dispatch";
import { getRecord } from "./agent-stream-records";
import { visibleAssistant } from "./agent-stream-visible-assistant";
import { resolvePreparedContextBuckets } from "./context-usage-stream";
import type { VisibleTurnIdentity } from "./agent-chat-stream-types";
import type { ChatStreamAdmission } from "@/types/agent-turn.generated";

const UUID_V4 = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

export function reconcileTurnAdmission(
  sessionId: string,
  admission: ChatStreamAdmission,
  optimisticUserMessageId?: string,
) {
  reconcileIdentity(sessionId, admission, optimisticUserMessageId);
}

export function reconcileTurnEvent(
  sessionId: string,
  identity: VisibleTurnIdentity,
) {
  reconcileIdentity(sessionId, identity);
}

function reconcileIdentity(
  sessionId: string,
  identity: VisibleTurnIdentity,
  optimisticUserMessageId?: string,
) {
  if (!validIdentity(identity)) return;
  const record = getRecord(sessionId);
  if (!record) return;
  const existing = record.state.messages.findIndex(
    (message) => message.id === identity.userMessageId,
  );
  const optimistic = existing >= 0
    ? existing
    : optimisticMessageIndex(record.state.messages, optimisticUserMessageId);
  let messages = record.state.messages;
  let queuedUserMessages = record.state.queuedUserMessages;
  if (optimistic >= 0) {
    messages = messages.map((message, index) => index === optimistic
      ? { ...message, id: identity.userMessageId, turn_id: identity.turnId }
      : message);
  } else if (queuedUserMessages.length > 0) {
    const [queued, ...remaining] = queuedUserMessages;
    messages = [...messages, {
      ...queued,
      id: identity.userMessageId,
      turn_id: identity.turnId,
    }];
    queuedUserMessages = remaining;
  }
  record.state = {
    ...record.state,
    messages,
    queuedUserMessages,
    activeTurn: identity,
    updatedAt: Date.now(),
  };
  flushFrameNotify(record, notifyRecord);
  notifyRecordActivity(sessionId, record);
}

function optimisticMessageIndex(
  messages: import("@/types/agent").AgentMessage[],
  expectedId?: string,
): number {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (message.role === "user" && (message.id === expectedId || !message.turn_id)) {
      return index;
    }
  }
  return -1;
}

export function reconcileTurnCommitted(
  sessionId: string,
  identity: VisibleTurnIdentity,
) {
  const record = getRecord(sessionId);
  if (!record || !sameIdentity(record.state.activeTurn, identity)) return;
  const assistant = visibleAssistant(record.state, null, true);
  record.state = {
    ...record.state,
    messages: assistant ? [...record.state.messages, assistant] : record.state.messages,
    contextUsageBuckets: resolvePreparedContextBuckets(record.state, []),
    contextUsageBaseSegments: 0,
    completedSegments: [],
    currentContent: "",
    currentContentPhase: undefined,
    currentThinking: "",
    currentTools: [],
    activeStreamItem: null,
    activeTurn: undefined,
    updatedAt: Date.now(),
  };
  flushFrameNotify(record, notifyRecord);
  notifyRecordActivity(sessionId, record);
}

function validIdentity(identity: VisibleTurnIdentity): boolean {
  return [identity.turnId, identity.userMessageId, identity.assistantMessageId]
    .every((value) => UUID_V4.test(value));
}

function sameIdentity(
  current: VisibleTurnIdentity | undefined,
  incoming: VisibleTurnIdentity,
): boolean {
  return current?.turnId === incoming.turnId
    && current.userMessageId === incoming.userMessageId
    && current.assistantMessageId === incoming.assistantMessageId;
}
