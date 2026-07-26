import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import i18n from "@/i18n";
import {
  applyStreamEvent,
  finishPartialStream,
} from "./agent-chat-stream-callbacks";
import {
  MAX_EVENTS_PER_SESSION,
  scheduleCleanup, clearCleanup, trimSubscribers,
} from "./agent-stream-cleanup";
import {
  flushFrameNotify,
  scheduleFrameNotify,
  shouldDeferStreamEvent,
} from "./agent-stream-notify";
import {
  acceptsStreamEvent,
  markStreamCancelled,
} from "./agent-stream-generations";
import {
  getOrCreateRecord,
  getRecord,
  persistMessages,
  records,
  snapshot,
  startStreamRecord,
  touchSession,
  type StreamSnapshot,
} from "./agent-stream-records";
import { subscribeStreamActivity } from "./agent-stream-activity";
import { getActivity, getSnapshot, isStreaming, setSessionGeneration } from "./agent-stream-access";
import { handleCompressionComplete } from "./agent-stream-compression-complete";
import {
  finishPersistenceRun,
  frontendShouldPersist,
  markIncomingBackendRun,
} from "./agent-stream-persistence-owner";
import { applySessionSnapshot } from "./agent-stream-snapshot";
import {
  notifyRecord as notify,
  notifyRecordActivity as notifyActivity,
} from "./agent-stream-notify-dispatch";
import { showToast } from "@/lib/toast-emitter";
import { clearStreamPermission } from "./agent-stream-permissions";
import type { AgentMessage, StreamEvent } from "@/types/agent";
import { webToolErrorToastMessage } from "./web-tool-error-toast";
import { queueUserMessage, removeQueuedUserMessage } from "./agent-stream-user-queue";
import { failSession } from "./agent-stream-failure";
import type { StreamKind } from "./agent-chat-stream-types";

export type { StreamSnapshot } from "./agent-stream-records";
const EVENT_NAME = "agent-stream-event";

interface StreamEnvelope { sessionId: string; generation?: number; event: StreamEvent }

type Subscriber = (snapshot: StreamSnapshot) => void;

let listenPromise: Promise<UnlistenFn> | null = null;

export const agentStreamManager = { startSession, stopSession, failSession, setSessionGeneration,
  clearPermission: clearStreamPermission, getSnapshot, getActivity, isStreaming, subscribe,
  queueUserMessage, removeQueuedUserMessage, subscribeActivity: subscribeStreamActivity };

function ensureListener() {
  if (!listenPromise) {
    listenPromise = listen<StreamEnvelope>(EVENT_NAME, (event) => {
      if (!event.payload?.sessionId) return;
      handleStreamEvent(
        event.payload.sessionId,
        event.payload.event,
        typeof event.payload.generation === "number" ? event.payload.generation : null,
      );
    });
  }
  return listenPromise;
}

async function startSession(
  sessionId: string,
  messages: AgentMessage[],
  sessionTokenCount: number,
  streamKind: StreamKind = "chat",
) {
  await ensureListener();
  const record = startStreamRecord(sessionId, messages, sessionTokenCount, streamKind);
  flushFrameNotify(record, notify);
  notifyActivity(sessionId, record);
}

function stopSession(sessionId: string, generation?: number | null) {
  const record = getRecord(sessionId);
  if (!record) return;
  markStreamCancelled(record, generation);
  const result = finishPartialStream(record.state);
  record.state = result.state;
  flushFrameNotify(record, notify);
  notifyActivity(sessionId, record);
  if (result.messagesToPersist && !record.state.persisted && frontendShouldPersist(record)) {
    persistMessages(sessionId, record, result.messagesToPersist, 0, true, notify);
  }
  finishPersistenceRun(record);
}

function subscribe(sessionId: string, subscriber: Subscriber): () => void {
  void ensureListener();
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);
  const id = record.nextSubscriberId++;
  record.subscribers.set(id, subscriber as (s: unknown) => void);
  trimSubscribers(record);
  if (record.started) subscriber(snapshot(record.state));
  return () => {
    record.subscribers.delete(id);
    if (record.state.completed && record.subscribers.size === 0) {
      scheduleCleanup(sessionId, record, records);
    }
  };
}

function handleStreamEvent(sessionId: string, event: StreamEvent, generation: number | null) {
  const record = getOrCreateRecord(sessionId);
  clearCleanup(record);

  markIncomingBackendRun(record);

  if (!acceptsStreamEvent(record, generation, event)) return;

  if (event.event === "subagentCompleted") {
    if (isStreaming(event.data.subagentSessionId)) stopSession(event.data.subagentSessionId);
    flushFrameNotify(record, notify);
    return;
  }

  if (event.event === "subagentSpawned" || event.event === "todoUpdated") {
    flushFrameNotify(record, notify);
    return;
  }

  if (event.event === "sessionSnapshot") {
    applySessionSnapshot(record, event.data.messages);
    flushFrameNotify(record, notify);
    notifyActivity(sessionId, record);
    return;
  }

  if (event.event === "notice") {
    showToast(i18n.t(event.data.messageKey), "info");
    return;
  }

  if (!record.state.isStreaming && event.event !== "done" && event.event !== "error") {
    record.state = { ...record.state, isStreaming: true, persisted: false, completed: false };
  }
  record.history.push(event);
  if (record.history.length > MAX_EVENTS_PER_SESSION) {
    record.history.splice(0, record.history.length - MAX_EVENTS_PER_SESSION);
  }

  const toastMessage = webToolErrorToastMessage(sessionId, event);
  if (toastMessage) showToast(toastMessage, "error");

  if (event.event === "compressionComplete") {
    handleCompressionComplete(sessionId, record, notify, notifyActivity);
    return;
  }

  const result = applyStreamEvent(record.state, event);
  record.state = result.state;
  if (record.state.completed) markStreamCancelled(record, generation);
  touchSession(sessionId, record);
  if (shouldDeferStreamEvent(event)) {
    scheduleFrameNotify(record, notify);
  } else {
    flushFrameNotify(record, notify);
  }
  notifyActivity(sessionId, record);

  if (result.messagesToPersist && !record.state.persisted && frontendShouldPersist(record)) {
    persistMessages(
      sessionId,
      record,
      result.messagesToPersist,
      result.assistantTokens ?? 0,
      record.state.completed,
      notify,
    );
  }
  if (record.state.completed) finishPersistenceRun(record);

  if (record.state.completed && record.subscribers.size === 0) {
    scheduleCleanup(sessionId, record, records);
  }
}
