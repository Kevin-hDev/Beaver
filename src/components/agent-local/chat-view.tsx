import { useState } from "react";
import { ChatMessagePanel } from "./chat-message-panel";
import { ChatInput } from "./chat-input";
import { ErrorBubble } from "./error-bubble";
import { FileDropZone } from "./file-drop-zone";
import { ChatOverlays } from "./chat-overlays";
import { SubagentAccordion } from "./subagent-accordion";
import { TodoProgressPanel } from "./todo-progress-panel";
import { ChatInputFooter } from "./chat-input-footer";
import { ChatTerminalDock } from "./chat-terminal-dock";
import { CloneSummaryRunButton } from "./clone-summary-run-button";
import { useAgentChat } from "@/hooks/use-agent-chat";
import { useContextProgress } from "@/hooks/use-context-progress";
import { useContextUsage } from "@/hooks/use-context-usage";
import { useFileDrop, type DroppedFile } from "@/hooks/use-file-drop";
import { usePermissionMode } from "@/hooks/use-permission-mode";
import { usePermissionRequests } from "@/hooks/use-permission-requests";
import { useSessionProject } from "@/hooks/use-session-project";
import { useChatScroll } from "@/hooks/use-chat-scroll";
import { useModelSwitch } from "@/hooks/use-model-switch";
import { useWorktreeSessionSwitch } from "@/hooks/use-worktree-session-switch";
import { useSessionFileGroups } from "@/hooks/use-session-files";
import { useSubagents } from "@/hooks/use-subagents";
import { useChatActions } from "@/hooks/use-chat-actions";
import { useChatClone } from "@/hooks/use-chat-clone";
import { useCloneGitBranchAction } from "@/hooks/use-clone-git-branch-action";
import { useSelectedModelCapabilities } from "@/hooks/use-selected-model-capabilities";
import { useChatViewRuntime } from "@/hooks/use-chat-view-runtime";
import { PermissionDialog } from "./permission-dialog";
import type { ChatViewProps } from "./chat-view-types";
import "./chat.css";
export function ChatView({
  sessionId, model, provider, projects, git, onAddProject,
  onSessionsRefresh, onApplySwitch, onNewSession, onNewSessionInProject, onAutoRename,
  initialMessage, initialWorkingDir, initialSkills, initialFiles,
  reasoningMode, onReasoningModeChange, onInitialMessageSent,
  terminalState, onFileOperationsChange, onFilePreviewPath,
  onOpenSubagent, isSubagent = false,
  canCloneMessages = false, onCloneMessage, onCancelCloneSummary,
  activeSessionTab, onCreateCloneGitBranch, onLinkCloneGitBranch,
}: ChatViewProps) {
  const permissions = usePermissionRequests();
  const permMode = usePermissionMode(sessionId);
  const selectedModelCaps = useSelectedModelCapabilities(provider, model);
  const chat = useAgentChat(sessionId, model, provider, (id, toolName, args) =>
    permissions.enqueue({ id, toolName, arguments: args }),
    selectedModelCaps?.supports_tools,
    selectedModelCaps?.supports_thinking,
    selectedModelCaps?.supports_vision,
    reasoningMode,
    permMode.mode,
    permMode.refresh,
  );
  const subagents = useSubagents(isSubagent ? undefined : sessionId);
  const knownSubagents = [...subagents.active, ...subagents.completed];
  const fileDrop = useFileDrop();
  const context = useContextProgress(model, chat.sessionTokenCount, provider);
  const [preview, setPreview] = useState<DroppedFile | null>(null);
  const proj = useSessionProject(sessionId, projects, onAddProject, chat.messages.length > 0);
  const contextUsage = useContextUsage({
    sessionId, model, provider, messages: chat.messages, used: context.used,
    workingDir: proj.selectedProject?.path, permissionMode: permMode.mode,
    planMode: chat.planModeEnabled, supportsTools: selectedModelCaps?.supports_tools,
  });
  useSessionFileGroups(
    chat.messages,
    chat.completedSegments,
    chat.currentTools,
    proj.selectedProject?.path,
    onFileOperationsChange,
  );
  const { handleSend, handleFileImport } = useChatActions({
    chat, selectedProjectPath: proj.selectedProject?.path,
    selectedProjectId: proj.selectedProjectId ?? undefined,
    onSessionsRefresh, onAutoRename, sessionId,
    initialMessage, initialWorkingDir, initialSkills, initialFiles,
    onInitialMessageSent, fileDrop,
  });
  const { containerRef, isAtBottom, scrollToBottom } = useChatScroll(
    sessionId, chat.isStreaming,
    [chat.currentContent, chat.currentContentPhase, chat.currentThinking, chat.completedSegments, chat.messages, chat.planPreview],
  );
  const clone = useChatClone(sessionId, chat.messages, onCloneMessage, onCancelCloneSummary);
  const cloneGitBranch = useCloneGitBranchAction({
    projectPath: proj.selectedProject?.path,
    git,
    isStreaming: chat.isStreaming,
    activeSessionTab,
    onCreateCloneGitBranch,
  });
  const runtime = useChatViewRuntime({
    chat,
    projectPath: proj.selectedProject?.path,
    activeSessionTab,
    onLinkCloneGitBranch,
    setPreview,
  });
  const { pendingSwitch, setPendingSwitch, handleModelSelect, rememberedRef } = useModelSwitch({
    currentModel: model, currentProvider: provider,
    messagesLength: chat.messages.length, onApplySwitch, onNewSession,
  });
  const worktreeSwitch = useWorktreeSessionSwitch({
    projects, model, provider, onAddProject, onNewSessionInProject,
  });
  return (
    <FileDropZone dragging={fileDrop.dragging} onDragChange={fileDrop.setDragging} onDropPaths={(paths) => void fileDrop.addByPaths(paths)}>
      <div className="chat-zone" style={{ opacity: chat.sessionLoading ? 0 : 1 }}>
        <div className="chat-messages" ref={containerRef}>
          <ChatMessagePanel
            chat={chat}
            runtime={runtime}
            projectPath={proj.selectedProject?.path}
            knownSubagents={knownSubagents}
            cloneEnabled={canCloneMessages && !!onCloneMessage}
            requestClone={clone.requestClone}
            onFilePreviewPath={onFilePreviewPath}
            onOpenSubagent={onOpenSubagent}
          />
        </div>
        <div className="chat-input-area">
          <div className="chat-input-column">
            <TodoProgressPanel sessionId={isSubagent ? undefined : sessionId} />
            {subagents.active.length > 0 && (
              <SubagentAccordion
                subagents={subagents.active}
                onCancel={(id) => void subagents.cancelSubagent(id)}
                onOpen={(id) => onOpenSubagent?.(id)}
              />
            )}
            {permissions.current && (
              <PermissionDialog request={permissions.current} onDecide={(id, decision) => void permissions.respond(id, decision)} />
            )}
            {runtime.showError && chat.error && (
              <ErrorBubble
                message={chat.error}
                isConnection={chat.isConnectionError}
                diagnosticSummary={chat.diagnosticSummary}
                onRetry={runtime.handleRetry}
              />
            )}
            <ChatInput
              modelName={model} providerName={provider} isStreaming={chat.isStreaming} reasoningMode={reasoningMode}
              files={fileDrop.files} contextUsed={contextUsage.used} contextMax={context.max} contextBreakdown={contextUsage}
              retryIndicator={runtime.retryIndicator}
              interactiveRequest={chat.interactiveChoice}
              onInteractiveResolved={chat.clearInteractiveChoice}
              permissionMode={permMode.mode}
              availablePermissionModes={permMode.availableModes}
              missingDirectory={chat.missingDirectory}
              missingDirectoryResolving={chat.missingDirectoryResolving}
              onPermissionModeChange={(m) => void permMode.change(m)}
              onResolveMissingDirectory={(action) => void chat.resolveMissingDirectory(action)}
              planModeEnabled={chat.planModeEnabled}
              onPlanModeChange={(enabled) => void chat.setPlanModeEnabled(enabled)}
              onRemoveFile={fileDrop.removeFile} onPreviewFile={setPreview} onSend={handleSend}
              onStop={() => void chat.stop()} onClearFiles={fileDrop.clearFiles} onFileImport={handleFileImport}
              onModelChange={handleModelSelect} onReasoningModeChange={onReasoningModeChange}
            />
            <ChatInputFooter
              projects={projects}
              projectState={proj}
              git={git}
              showScrollBottom={!isAtBottom}
              centerSlot={clone.summaryRun && !clone.summaryRun.visible
                ? <CloneSummaryRunButton onClick={clone.showRunningClone} />
                : null}
              onScrollBottom={scrollToBottom}
              onWorktreeSelect={worktreeSwitch.request}
              onBranchReady={runtime.handleBranchReady}
              cloneGitBranch={cloneGitBranch}
            />
          </div>
        </div>
        <ChatTerminalDock terminalState={terminalState} />
      </div>
      <ChatOverlays
        preview={preview} currentModel={model} pendingSwitch={pendingSwitch}
        pendingWorktreeSwitch={worktreeSwitch.pending}
        pendingClone={clone.pendingClone}
        cloneBusy={clone.cloneBusy}
        onClosePreview={() => setPreview(null)} onCancelSwitch={() => setPendingSwitch(null)}
        onCancelWorktreeSwitch={worktreeSwitch.cancel}
        onCancelClone={clone.cancelClone}
        onAbortClone={() => void clone.abortClone()}
        onSubmitClone={(mode, customFocus) => void clone.submitClone(mode, customFocus)}
        onNewSession={(remember) => { if (remember) rememberedRef.current = "new"; onNewSession?.(pendingSwitch!.model, pendingSwitch!.provider); setPendingSwitch(null); }}
        onContinue={(remember) => { if (remember) rememberedRef.current = "continue"; onApplySwitch?.(pendingSwitch!.model, pendingSwitch!.provider); setPendingSwitch(null); }}
        onNewWorktreeSession={() => void worktreeSwitch.createSession()}
      />
    </FileDropZone>
  );
}
