import { useMemo, memo } from "react";
import { ChatHeader } from "./chat-header";
import { AgentChatDetail } from "./agent-chat-detail";
import { WelcomeView } from "./welcome-view";
import { useAgentLocalTab } from "@/hooks/use-agent-local-tab";
import { useFileTree } from "@/hooks/use-file-tree";
import { useForecastPanel } from "@/hooks/use-forecast-panel";
import { useAgentLocalPanelNav } from "@/hooks/use-agent-local-panel-nav";
import { useAgentLocalControlledPanels } from "@/hooks/use-agent-local-controlled-panels";
import { useGitBranch } from "@/hooks/use-git-branch";
import { useGitUncommittedFiles } from "@/hooks/use-git-uncommitted-files";
import { useSessionSummary } from "@/hooks/use-session-summary";
import { useSessionTabs } from "@/hooks/use-session-tabs";
import { useAgentLocalTabGit } from "@/hooks/use-agent-local-tab-git";
import { PanelSlot } from "@/components/layout/panel-slots";
import {
  resolveDisplayModel, resolveDisplayProject, resolveDisplayReasoningMode, resolveDisplaySession,
} from "./agent-local-display";
import { useAgentLocalForecastContent } from "./use-agent-local-forecast-content";
import { useAgentLocalConversationList } from "./use-agent-local-conversation-list";
import { useAvailablePanelMode } from "@/hooks/use-available-panel-mode";
import { commitFileOperation } from "@/lib/git-file-preview";
import type { AgentLocalTabProps } from "./agent-local-tab-types";
import "./agent-local-tab.css";

export const AgentLocalTab = memo(function AgentLocalTab({
  navState,
  onSessionChange,
  onNavChange,
  listFocused = true,
}: AgentLocalTabProps) {
  const s = useAgentLocalTab({ navState, onSessionChange, onNavChange, listFocused });
  const { sessions, refresh, archive, updateModel } = s;
  const { projectsHook, terminal, activeSession, activeSessionId } = s;
  const { model, provider, currentDefault, activeProject } = s;
  const { filePreview, fileOperations, setFileOperations } = s;
  const { reasoningMode, setReasoningMode, setWelcomeModel } = s;
  const { sessionActions, handleSelectById } = s;
  const {
    pendingMessage, setPendingMessage,
    pendingWorkingDir, setPendingWorkingDir,
    pendingSkills, setPendingSkills,
    pendingFiles, setPendingFiles,
    handleCreateWithModel,
    handleWelcomeSend, handleAutoRename,
    handleCreateInProjectWithModel,
  } = sessionActions;
  const sessionTabs = useSessionTabs(activeSessionId, refresh);
  const displaySessionId = sessionTabs.activeSessionId ?? activeSessionId;
  const displaySession = resolveDisplaySession(sessions, displaySessionId, activeSession);
  const displayProject = resolveDisplayProject(projectsHook.projects, displaySession, activeProject);
  const { displayModel, displayProvider } = resolveDisplayModel(displaySession, model, provider);
  const displayReasoningMode = resolveDisplayReasoningMode(displaySession, reasoningMode);
  const terminalCwd = displayProject?.path || "";
  const sessionSummary = useSessionSummary(displaySessionId ?? null);
  const summaryGit = useGitBranch(displayProject?.path);
  const uncommittedFiles = useGitUncommittedFiles(summaryGit);
  const tabGit = useAgentLocalTabGit({
    rootSessionId: activeSessionId,
    git: summaryGit,
    projectPath: displayProject?.path,
    sessionTabs,
  });
  const fileTree = useFileTree(displaySessionId, displayProject?.path);
  const forecast = useForecastPanel(displaySessionId ?? null);
  useAgentLocalPanelNav({ navState, fileTree, forecast });
  const { fileTreeNav, forecastNav } = useAgentLocalControlledPanels({
    navState, sessionId: displaySessionId ?? null, filePreview, fileTree, forecast, onNavChange,
  });
  const availablePanel = useAvailablePanelMode(forecastNav.panelMode);
  const {
    forecastContent, fullscreenSwitching, handleOpenForecastDocs, handlePreviewFullscreenChange,
  } = useAgentLocalForecastContent({
    forecastNav,
    filePreview,
    sessionId: displaySessionId ?? null,
  });

  const list = useAgentLocalConversationList(s, activeSessionId);

  const detail = useMemo(() => (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      <div style={{ flexShrink: 0 }}>
        <ChatHeader
          sessionName={activeSession?.name ?? null}
          sessionId={activeSessionId ?? null}
          terminalOpen={terminal.isOpen}
          previewOpen={filePreview.open}
          panelMode={availablePanel.panelMode}
          browserStatus={availablePanel.browserStatus}
          showForecastDocs={filePreview.open && availablePanel.panelMode === "forecast"}
          onTogglePreview={filePreview.toggleOpen}
          onOpenForecastDocs={handleOpenForecastDocs}
          onPanelModeChange={forecastNav.setPanelMode}
          sessionSummary={sessionSummary}
          summaryGit={summaryGit}
          sessionTabs={sessionTabs.tabs}
          sessionTabAttentionIds={sessionTabs.attentionTabIds}
          onSelectSessionTab={(id) => void tabGit.selectTab(id)}
          onCloseSessionTab={tabGit.closeTab}
          onRenameSessionTab={(id, label) => void sessionTabs.renameTab(id, label)}
          onOpenPlan={filePreview.openPlan}
          onOpenSubagent={(id) => void handleSelectById(id)}
          onArchiveSubagent={(id) => void archive(id)}
          onOpenGitFile={(commit, file) => filePreview.openOperation(
            commitFileOperation(commit, file, summaryGit.currentBranch),
          )}
          onToggleTerminal={() => {
            if (!terminal.isOpen && terminal.tabs.length === 0) {
              terminal.addTab(terminalCwd);
            } else {
              terminal.togglePanel();
            }
          }}
        />
      </div>
      {displaySessionId ? (
        <AgentChatDetail
          sessionId={displaySessionId}
          model={displayModel}
          provider={displayProvider}
          projects={projectsHook.projects}
          activeProjectPath={displayProject?.path}
          pendingMessage={pendingMessage}
          pendingWorkingDir={pendingWorkingDir}
          pendingSkills={pendingSkills}
          pendingFiles={pendingFiles}
          reasoningMode={displayReasoningMode}
          terminal={terminal}
          filePreview={filePreview}
          fullscreenSwitching={fullscreenSwitching}
          fileOperations={fileOperations}
          gitUncommittedFiles={uncommittedFiles}
          git={summaryGit}
          fileTree={fileTreeNav}
          onAddProject={projectsHook.add}
          onSessionsRefresh={() => void refresh()}
          onUpdateModel={(id, m, p) => void updateModel(id, m, p)}
          onNewSession={(m, p) => void handleCreateWithModel(m, p)}
          onNewSessionInProject={(m, p, pid) =>
            void handleCreateInProjectWithModel(m, p, pid)}
          onAutoRename={(id, msg) => void handleAutoRename(id, msg)}
          onReasoningModeChange={setReasoningMode}
          onInitialMessageSent={() => {
            setPendingMessage(null);
            setPendingWorkingDir(undefined);
            setPendingSkills(undefined);
            setPendingFiles(undefined);
          }}
          onFileOperationsChange={setFileOperations}
          onPreviewFullscreenChange={handlePreviewFullscreenChange}
          panelMode={availablePanel.panelMode}
          forecastContent={forecastContent}
          parentSessionId={displaySession?.parent_session_id}
          onOpenSubagent={(id) => void handleSelectById(id)}
          onGoToParent={() => displaySession?.parent_session_id && void handleSelectById(displaySession.parent_session_id)}
          canCloneMessages={!displaySession?.parent_session_id}
          onCloneMessage={(messageId, cloneMode, customFocus, options) =>
            sessionTabs.cloneMessage({
              messageId,
              mode: cloneMode,
              customFocus,
              operationId: options?.operationId,
              shouldActivateOnComplete: options?.shouldActivateOnComplete,
            }).then(() => undefined)}
          onCancelCloneSummary={(operationId) => sessionTabs.cancelCloneSummary(operationId)}
          activeSessionTab={sessionTabs.activeTab}
          onCreateCloneGitBranch={sessionTabs.createCloneGitBranch}
          onLinkCloneGitBranch={sessionTabs.linkCloneGitBranch}
        />
      ) : (
        <div style={{ flex: 1, minHeight: 0, overflow: "hidden" }}>
          <WelcomeView
            model={currentDefault.model}
            provider={currentDefault.provider}
            projects={projectsHook.projects}
            onAddProject={projectsHook.add}
            onSend={(...args) => void handleWelcomeSend(...args)}
            onModelChange={(m, p) => setWelcomeModel({ model: m, provider: p })}
            reasoningMode={reasoningMode}
            onReasoningModeChange={setReasoningMode}
          />
        </div>
      )}
    </div>
  ), [
    activeSession?.name, activeSessionId, archive, currentDefault.model, currentDefault.provider, displayModel, displayProject?.path,
    displayProvider, displayReasoningMode, displaySession?.parent_session_id,
    displaySessionId,
    availablePanel, fileOperations, filePreview, fileTreeNav, forecastNav.setPanelMode, forecastContent, uncommittedFiles,
    fullscreenSwitching, handleAutoRename, handleCreateInProjectWithModel, handleCreateWithModel,
    handleOpenForecastDocs, handlePreviewFullscreenChange, handleSelectById, handleWelcomeSend,
    pendingFiles, pendingMessage, pendingSkills, pendingWorkingDir, projectsHook, refresh,
    sessionSummary, sessionTabs, setFileOperations, setPendingFiles, setPendingMessage, setPendingSkills, setPendingWorkingDir, setReasoningMode,
    setWelcomeModel, summaryGit, tabGit, terminal, terminalCwd, reasoningMode, updateModel,
  ]);

  return <><PanelSlot name="list">{list}</PanelSlot><PanelSlot name="detail">{detail}</PanelSlot>{tabGit.dialogs}</>;
});
