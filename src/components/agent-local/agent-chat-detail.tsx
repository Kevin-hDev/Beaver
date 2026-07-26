import type { CSSProperties } from "react";
import { ChatView } from "./chat-view";
import { AgentSidePanel } from "@/components/agent-side-panel/agent-side-panel";
import { FilePreviewPanel } from "@/components/file-preview/file-preview-panel";
import { FileTreePanel } from "@/components/file-tree/file-tree-panel";
import { BrowserPanel } from "@/components/internal-browser/browser-panel";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { useFileTree } from "@/hooks/use-file-tree";
import { useAgentPanelLayout } from "@/hooks/use-agent-panel-layout";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { useTerminal } from "@/hooks/use-terminal";
import type { CloneMessageHandler } from "@/hooks/use-chat-clone";
import type { Project, SessionTab } from "@/types/agent";
import type { FileOperation, FileOperationGroups } from "@/types/file-preview";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import type { useGitBranch } from "@/hooks/use-git-branch";

interface AgentChatDetailProps {
  sessionId: string;
  model: string;
  provider: string;
  projects: Project[];
  activeProjectPath?: string;
  pendingMessage?: string | null;
  pendingWorkingDir?: string;
  pendingSkills?: { name: string; content: string }[];
  pendingFiles?: DroppedFile[];
  reasoningMode?: string | null;
  terminal: ReturnType<typeof useTerminal>;
  filePreview: ReturnType<typeof useFilePreview>;
  fullscreenSwitching: boolean;
  fileOperations: FileOperationGroups;
  gitUncommittedFiles: FileOperation[];
  git: ReturnType<typeof useGitBranch>;
  onAddProject: (path: string) => Promise<Project>;
  onSessionsRefresh: () => void;
  onUpdateModel: (id: string, model: string, provider: string) => void;
  onNewSession: (model: string, provider: string) => void;
  onNewSessionInProject: (model: string, provider: string, projectId: string) => void;
  onAutoRename: (id: string, name: string) => void;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  onInitialMessageSent: () => void;
  onFileOperationsChange: (operations: FileOperationGroups) => void;
  onPreviewFullscreenChange: (fullscreen: boolean) => void;
  fileTree: ReturnType<typeof useFileTree>;
  parentSessionId?: string;
  onOpenSubagent?: (sessionId: string) => void;
  onGoToParent?: () => void;
  canCloneMessages?: boolean;
  onCloneMessage?: CloneMessageHandler;
  onCancelCloneSummary?: (operationId: string) => Promise<void>;
  activeSessionTab?: SessionTab | null;
  onCreateCloneGitBranch?: (path: string, cloneSessionId: string) => Promise<string>;
  onLinkCloneGitBranch?: (path: string, cloneSessionId: string, branchName: string) => Promise<void>;
  panelMode?: PanelMode;
  forecastContent?: React.ReactNode;
}

export function AgentChatDetail(props: AgentChatDetailProps) {
  const previewDesiredWidth = props.filePreview.width + props.filePreview.extraWidth;
  const previewFullscreen = props.panelMode !== "forecast" && props.filePreview.fullscreen;
  const openPreviewTarget = (target: string | FileOperation) => (
    typeof target === "string" ? props.filePreview.openPath(target) : props.filePreview.openOperation(target)
  );
  const { containerRef, layout } = useAgentPanelLayout({
    previewOpen: props.filePreview.open,
    previewFullscreen,
    previewDesiredWidth,
    fileTreeOpen: props.fileTree.open,
    fileTreeDesiredWidth: props.fileTree.width,
  });
  const layoutStyle = {
    "--agent-chat-min-width": `${layout.chatMinWidth}px`,
    "--ft-active-width": `${props.fileTree.open ? layout.fileTreeWidth : 0}px`,
  } as CSSProperties;

  return (
    <div className="agent-detail-with-preview" ref={containerRef} style={layoutStyle}>
      {props.parentSessionId && (
        <button
          className="sa-parent-btn"
          onClick={props.onGoToParent}
          type="button"
        >
          ← Chat parent
        </button>
      )}
      <div className={`agent-detail-chat ${previewFullscreen ? "agent-detail-chat-fs" : ""} ${props.fullscreenSwitching ? "agent-detail-chat-instant" : ""}`}>
        <ChatView
          sessionId={props.sessionId}
          model={props.model}
          provider={props.provider}
          projects={props.projects}
          git={props.git}
          onAddProject={props.onAddProject}
          onSessionsRefresh={props.onSessionsRefresh}
          onApplySwitch={(model, provider) => props.onUpdateModel(props.sessionId, model, provider)}
          onNewSession={props.onNewSession}
          onNewSessionInProject={props.onNewSessionInProject}
          onAutoRename={props.onAutoRename}
          initialMessage={props.pendingMessage ?? undefined}
          initialWorkingDir={props.pendingWorkingDir}
          initialSkills={props.pendingSkills}
          initialFiles={props.pendingFiles}
          reasoningMode={props.reasoningMode}
          onReasoningModeChange={props.onReasoningModeChange}
          onInitialMessageSent={props.onInitialMessageSent}
          terminalState={props.terminal}
          onFileOperationsChange={props.onFileOperationsChange}
          onFilePreviewPath={openPreviewTarget}
          onOpenSubagent={props.onOpenSubagent}
          isSubagent={!!props.parentSessionId}
          canCloneMessages={props.canCloneMessages}
          onCloneMessage={props.onCloneMessage}
          onCancelCloneSummary={props.onCancelCloneSummary}
          activeSessionTab={props.activeSessionTab}
          onCreateCloneGitBranch={props.onCreateCloneGitBranch}
          onLinkCloneGitBranch={props.onLinkCloneGitBranch}
        />
      </div>
      <AgentSidePanel
        open={props.filePreview.open}
        fullscreen={previewFullscreen}
        displayWidth={layout.previewWidth}
        fullscreenSwitching={props.fullscreenSwitching}
        resizing={props.filePreview.resizing}
        onResizeStart={props.filePreview.startResize}
        mode={props.panelMode ?? "preview"}
        previewContent={(
          <FilePreviewPanel
            fullscreen={props.filePreview.fullscreen}
            allOperations={props.fileOperations.all}
            latestOperations={props.fileOperations.latest}
            uncommittedOperations={props.gitUncommittedFiles}
            tabs={props.filePreview.tabs}
            activeTab={props.filePreview.activeTab}
            listMode={props.filePreview.listMode}
            baseDir={props.activeProjectPath}
            onFullscreenChange={props.onPreviewFullscreenChange}
            onActiveTabChange={props.filePreview.setActiveTab}
            onListModeChange={props.filePreview.setListMode}
            onOpenOperation={props.filePreview.openOperation}
            onOpenFilePath={props.filePreview.openFullPath}
            onCloseTab={props.filePreview.closeTab}
            hasProject={props.fileTree.hasProject}
            treeOpen={props.fileTree.open}
            onToggleTree={props.fileTree.toggleOpen}
          />
        )}
        forecastContent={props.forecastContent}
        browserContent={(
          <BrowserPanel
            conversationId={props.sessionId}
            active={props.filePreview.open && props.panelMode === "browser" && !props.fullscreenSwitching}
            fullscreen={props.filePreview.fullscreen}
            onFullscreenChange={props.onPreviewFullscreenChange}
          />
        )}
      />
      <FileTreePanel
        tree={props.fileTree}
        displayWidth={layout.fileTreeWidth}
        onFileSelect={props.filePreview.openPath}
        activePath={props.filePreview.tabs.find((tab) => tab.id === props.filePreview.activeTab)?.path ?? null}
      />
    </div>
  );
}
