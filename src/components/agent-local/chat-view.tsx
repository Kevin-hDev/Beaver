import { useRef, useState } from "react";
import { ChatInput } from "./chat-input";
import { ScrollBottomButton } from "./scroll-bottom-button";
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
import { usePreflightDirectoryAccessPrompt } from "@/hooks/use-preflight-directory-access-prompt";
import { useComposerHandoff } from "@/hooks/use-composer-handoff";
import { hasComposerPosition } from "@/lib/composer-handoff";
import { cn } from "@/lib/utils";
import { sessionComposerDraftKey } from "@/hooks/use-composer-draft";
import { PermissionDialog } from "./permission-dialog";
import { ChatTranscript } from "./chat-transcript";
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
  const permMode = usePermissionMode(sessionId, !isSubagent);
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
  const fileDrop = useFileDrop();
  const context = useContextProgress(model, chat.sessionTokenCount, provider);
  const contextMax = chat.contextLimitTokens || context.max;
  const [preview, setPreview] = useState<DroppedFile | null>(null);
  const proj = useSessionProject(sessionId, projects, onAddProject, chat.messages.length > 0);
  const contextUsage = useContextUsage({
    sessionId, model, provider, messages: chat.messages,
    stream: chat,
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
    readOnly: isSubagent,
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
    readOnly: isSubagent,
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
  const preflightAccessPrompt = usePreflightDirectoryAccessPrompt(chat.forbiddenAllowedPaths, chat.dismissForbiddenDirectory);
  /* Une conversation qui vient d'être créée depuis l'accueil n'a rien à charger :
     elle se montre dès son premier rendu. Attendre la fin de la lecture du
     disque laissait l'écran vide entre le champ qui part et celui qui arrive,
     et cette substitution se voyait. */
  const [handingOver] = useState(!isSubagent && hasComposerPosition);
  const visible = handingOver || !chat.sessionLoading;
  /* Le champ ne descend qu'une fois la conversation peinte : lancé pendant
     qu'elle est encore transparente, le glissement se jouerait à l'abri des
     regards et le champ paraîtrait surgir à sa place. */
  const inputColumnRef = useRef<HTMLDivElement>(null);
  useComposerHandoff(inputColumnRef, visible);
  return (
    <FileDropZone
      enabled={!isSubagent}
      dragging={fileDrop.dragging}
      onDragChange={fileDrop.setDragging}
      onDropPaths={(paths) => void fileDrop.addByPaths(paths)}
    >
      <div
        className={cn("chat-zone", isSubagent && "chat-zone-read-only")}
        style={{ opacity: visible ? 1 : 0 }}
      >
        <ChatTranscript
          chat={chat}
          cloneEnabled={canCloneMessages && !!onCloneMessage}
          containerRef={containerRef}
          isAtBottom={isAtBottom}
          knownSubagents={subagents.active.concat(subagents.completed)}
          onFilePreviewPath={onFilePreviewPath}
          onOpenSubagent={onOpenSubagent}
          onScrollBottom={scrollToBottom}
          projectPath={proj.selectedProject?.path}
          readOnly={isSubagent}
          requestClone={clone.requestClone}
          runtime={runtime}
        />
        {!isSubagent && (
          <div className="chat-input-area">
            <div className="chat-input-column" ref={inputColumnRef}>
              <TodoProgressPanel sessionId={sessionId} />
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
              <div className="chat-input-anchor">
                {!isAtBottom && <ScrollBottomButton onClick={scrollToBottom} />}
                <ChatInput
                  draftKey={sessionComposerDraftKey(sessionId)}
                  modelName={model} providerName={provider} isStreaming={chat.isStreaming} reasoningMode={reasoningMode}
                  files={fileDrop.files} contextUsed={contextUsage.used}
                  contextMax={chat.contextUsageVisible ? contextMax : 0} contextBreakdown={contextUsage}
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
              </div>
              <ChatInputFooter
                projects={projects}
                projectState={proj}
                git={git}
                centerSlot={clone.summaryRun && !clone.summaryRun.visible
                  ? <CloneSummaryRunButton onClick={clone.showRunningClone} />
                  : null}
                onWorktreeSelect={worktreeSwitch.request}
                directoryAccessPrompt={proj.directoryAccessPrompt ?? worktreeSwitch.directoryAccessPrompt ?? preflightAccessPrompt}
                onBranchReady={runtime.handleBranchReady}
                cloneGitBranch={cloneGitBranch}
              />
            </div>
          </div>
        )}
        {!isSubagent && <ChatTerminalDock terminalState={terminalState} />}
      </div>
      <ChatOverlays
        preview={preview} currentModel={model} pendingSwitch={pendingSwitch}
        readOnly={isSubagent}
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
