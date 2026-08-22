import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAgentStream } from "./use-agent-stream";
import { useAgentPlanMode } from "./use-agent-plan-mode";
import { useAgentPermissionDelivery } from "./use-agent-permission-delivery";
import { listenGatewaySessionUpdates } from "./use-gateway-session-updates";
import { clearInteractiveChoiceState, EMPTY_CHAT_STATE, type ChatState } from "./agent-chat-stream-callbacks";
import { resolveSessionContext } from "./agent-token-estimate";
import { createEditedUserMessage } from "./agent-message-builders";
import { useAgentMissingDirectory } from "./use-agent-missing-directory";
import { useAgentMessageSend } from "./use-agent-message-send";
import { showToast } from "@/lib/toast-emitter";
import { admissionErrorMessage } from "@/lib/admission-error";
import { restoredFailureState } from "./agent-chat-restored-failure";
import i18n from "@/i18n";
import type { AgentMessage, AgentSession } from "@/types/agent";
export function useAgentChat(
  sessionId: string | null,
  model: string,
  provider: string,
  onPermissionRequest?: (id: string, toolName: string, args: Record<string, unknown>) => void,
  supportsTools?: boolean,
  supportsThinking?: boolean,
  supportsVision?: boolean,
  reasoningMode?: string | null,
  permissionMode?: string,
  onStreamStarted?: () => void | Promise<void>,
) {
  const [state, setState] = useState<ChatState>(EMPTY_CHAT_STATE);
  const planMode = useAgentPlanMode(sessionId, setState);
  const missingDirectory = useAgentMissingDirectory(sessionId);
  const {
    missingDirectory: missingDirectoryState,
    resolving: missingDirectoryResolving,
    runOrDefer,
    resolve: resolveMissingDirectory,
    forbiddenAllowedPaths,
    dismissForbidden,
  } = missingDirectory;
  const {
    enabled: planModeEnabled,
    reset: resetPlanMode,
    applySession: applyPlanSession,
    applyStreamEnabled: applyPlanStreamEnabled,
    setEnabled: setPlanModeEnabled,
  } = planMode;
  const [sessionLoading, setSessionLoading] = useState(true);
  const savingRef = useRef(false);
  const sessionRef = useRef(sessionId);
  const permissions = useAgentPermissionDelivery(onPermissionRequest);
  const {
    startStream, queueStreamMessage, stopStream, subscribeToStream, getStreamSnapshot,
  } = useAgentStream();
  // eslint-disable-next-line react-hooks/refs -- callback capture pattern for stable closures
  sessionRef.current = sessionId;
  const reasoningModeRef = useRef(reasoningMode);
  // eslint-disable-next-line react-hooks/refs -- callback capture pattern for stable closures
  reasoningModeRef.current = reasoningMode;
  const permModeRef = useRef(permissionMode);
  // eslint-disable-next-line react-hooks/refs -- callback capture pattern for stable closures
  permModeRef.current = permissionMode;
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- reset on session change + fetch→setState are intentional
    setSessionLoading(true);
    setState(EMPTY_CHAT_STATE);
    resetPlanMode();
    permissions.clear();
    if (!sessionId) return;

    let alive = true;
    const applySnapshot = (snapshot: ReturnType<typeof getStreamSnapshot>) => {
      if (!snapshot || !alive || sessionRef.current !== sessionId) return;
      const { pendingPermissions, completed: _completed, ...chatState } = snapshot;
      setState(chatState);
      applyPlanStreamEnabled(chatState.planModeEnabled);
      setSessionLoading(false);
      for (const request of pendingPermissions) {
        permissions.deliver(request.id, request.toolName, request.arguments);
      }
    };
    const unsubscribe = subscribeToStream(sessionId, applySnapshot);
    applySnapshot(getStreamSnapshot(sessionId));
    invoke<AgentSession>("get_agent_session", { id: sessionId })
      .then((session) => {
        if (!alive || sessionRef.current !== sessionId) return;
        applyPlanSession(session);
        const snapshot = getStreamSnapshot(sessionId);
        if (snapshot && snapshot.messages.length >= session.messages.length) {
          applySnapshot(snapshot);
          return;
        }
        setState((s) => ({
          ...s,
          messages: session.messages,
          ...resolveSessionContext(session),
          ...restoredFailureState(session),
        }));
        setSessionLoading(false);
      })
      .catch((e: unknown) => { console.warn("Session load:", e); setSessionLoading(false); });
    const stopGatewayListener = listenGatewaySessionUpdates(sessionId, sessionRef, (session) => {
      setState((s) => ({
        ...s,
        messages: session.messages,
        ...resolveSessionContext(session),
        ...restoredFailureState(session),
      }));
    });
    return () => {
      alive = false;
      unsubscribe();
      stopGatewayListener();
    };
  }, [
    sessionId, subscribeToStream, getStreamSnapshot, permissions,
    resetPlanMode, applyPlanStreamEnabled, applyPlanSession,
  ]);
  const doStream = useCallback(async (
    llmMsgs: AgentMessage[],
    displayMsgs: AgentMessage[],
    streamSession: string,
    workingDir?: string,
    baseTokenCountOverride?: number,
    permissionMode?: string,
  ) => {
    await startStream(
      streamSession,
      model,
      provider,
      llmMsgs,
      reasoningModeRef.current !== "off" && !!reasoningModeRef.current,
      { displayMessages: displayMsgs, baseTokenCount: baseTokenCountOverride ?? state.sessionTokenCount },
      workingDir,
      supportsTools,
      supportsThinking,
      supportsVision,
      reasoningModeRef.current,
      permissionMode,
      planModeEnabled,
    );
    await onStreamStarted?.();
  }, [model, onStreamStarted, planModeEnabled, provider, startStream, state.sessionTokenCount, supportsTools, supportsThinking, supportsVision]);
  const sendMessage = useAgentMessageSend({
    sessionId,
    messages: state.messages,
    permissionModeRef: permModeRef,
    savingRef,
    runOrDefer,
    doStream,
    queueStreamMessage,
  });
  const syncTokenCount = useCallback(async (): Promise<number> => {
    if (!sessionId) return state.sessionTokenCount;
    const session = await invoke<AgentSession>("get_agent_session", { id: sessionId }).catch(() => null);
    if (session) {
      const context = resolveSessionContext(session);
      setState((s) => ({ ...s, ...context }));
      return context.sessionTokenCount;
    }
    return state.sessionTokenCount;
  }, [sessionId, state.sessionTokenCount]);

  const reload = useCallback(async (messageId: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    try {
      await invoke("truncate_and_replace_at", { sessionId, messageId, replacement: null });
    } catch (error) {
      showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
      return;
    }
    const freshTokenCount = await syncTokenCount();
    const msgs = state.messages.slice(0, idx + 1);
    await doStream(msgs, msgs, sessionId, undefined, freshTokenCount, permModeRef.current);
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const edit = useCallback(async (messageId: string, newContent: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    const newMsg = createEditedUserMessage(state.messages[idx], newContent);
    try {
      await invoke("truncate_and_replace_at", { sessionId, messageId, replacement: newMsg });
    } catch (error) {
      showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
      return;
    }
    const freshTokenCount = await syncTokenCount();
    const msgs = [...state.messages.slice(0, idx), newMsg];
    await doStream(msgs, msgs, sessionId, undefined, freshTokenCount, permModeRef.current);
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const stop = useCallback(async () => {
    if (sessionId) await stopStream(sessionId);
    setState((s) => ({ ...s, isStreaming: false }));
  }, [sessionId, stopStream]);

  const clearInteractiveChoice = useCallback(() => setState(clearInteractiveChoiceState), []);
  const ready = state.messages.length > 0 || !sessionId;

  return {
    ...state, ready, sessionLoading,
    planModeEnabled, setPlanModeEnabled,
    missingDirectory: missingDirectoryState,
    missingDirectoryResolving,
    resolveMissingDirectory,
    forbiddenAllowedPaths,
    dismissForbiddenDirectory: dismissForbidden,
    sendMessage, reload, edit, stop, clearInteractiveChoice,
  };
}
