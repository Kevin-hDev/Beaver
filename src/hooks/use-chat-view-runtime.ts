import { useCallback } from "react";
import { useOllamaConnectionRetry } from "@/hooks/use-ollama-connection-retry";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { AgentMessage } from "@/types/agent-message";
import type { RetryIndicatorState } from "@/types/agent";
import type { SessionTab } from "@/types/agent-session";

interface Params {
  chat: {
    messages: AgentMessage[];
    reload: (id: string) => void | Promise<void>;
    edit: (id: string, content: string) => void | Promise<void>;
    error?: string;
    isConnectionError?: boolean;
    isStreaming: boolean;
    retryIndicator?: RetryIndicatorState | null;
  };
  projectPath?: string;
  activeSessionTab?: SessionTab | null;
  onLinkCloneGitBranch?: (
    projectPath: string,
    sessionId: string,
    branchName: string,
  ) => Promise<void>;
  setPreview: (file: DroppedFile) => void;
}

export function useChatViewRuntime(params: Params) {
  const {
    projectPath, activeSessionTab, onLinkCloneGitBranch, setPreview,
  } = params;
  const {
    messages, reload, edit, error, isConnectionError, isStreaming, retryIndicator,
  } = params.chat;
  const handleRetry = useCallback(() => {
    const message = [...messages].reverse().find((item) => item.role === "user");
    if (message) void reload(message.id);
  }, [messages, reload]);
  const connectionRetry = useOllamaConnectionRetry({
    error,
    isConnectionError,
    isStreaming,
    onRetry: handleRetry,
  });
  const handleBranchReady = useCallback(async (branchName: string) => {
    if (!projectPath || !activeSessionTab || activeSessionTab.is_main || activeSessionTab.git_branch) return;
    await onLinkCloneGitBranch?.(projectPath, activeSessionTab.session_id, branchName);
  }, [activeSessionTab, onLinkCloneGitBranch, projectPath]);
  const handleReload = useCallback((id: string) => void reload(id), [reload]);
  const handleEdit = useCallback(
    (id: string, content: string) => void edit(id, content),
    [edit],
  );
  const handleFileClick = useCallback((file: {
    name: string;
    path?: string;
    thumbnail?: string;
    access_grant?: string;
  }) => {
    setPreview({
      name: file.name,
      path: file.path,
      type: "",
      size: 0,
      preview: file.thumbnail,
      accessGrant: file.access_grant,
    });
  }, [setPreview]);

  return {
    handleBranchReady,
    handleRetry,
    handleReload,
    handleEdit,
    handleFileClick,
    retryIndicator: retryIndicator ?? connectionRetry.indicator,
    showError: !!error && !isStreaming && !connectionRetry.suppressError,
  };
}
