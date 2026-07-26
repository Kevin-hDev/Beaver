import type { CloneMessageHandler } from "@/hooks/use-chat-clone";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { useTerminal } from "@/hooks/use-terminal";
import type { Project, SessionTab } from "@/types/agent";
import type { FileOperation, FileOperationGroups } from "@/types/file-preview";
import type { ReasoningMode } from "@/lib/reasoning-modes";
import type { useGitBranch } from "@/hooks/use-git-branch";

export interface ChatViewProps {
  sessionId: string;
  model: string;
  provider: string;
  projects: Project[];
  git: ReturnType<typeof useGitBranch>;
  onAddProject: (path: string) => Promise<Project>;
  onSessionsRefresh?: () => void;
  onApplySwitch?: (model: string, provider: string) => void;
  onNewSession?: (model: string, provider: string) => void;
  onNewSessionInProject?: (model: string, provider: string, projectId: string) => void;
  onAutoRename?: (id: string, name: string) => void;
  initialMessage?: string;
  initialWorkingDir?: string;
  initialSkills?: { name: string; content: string }[];
  initialFiles?: DroppedFile[];
  reasoningMode?: string | null;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  onInitialMessageSent?: () => void;
  terminalState: ReturnType<typeof useTerminal>;
  onFileOperationsChange?: (operations: FileOperationGroups) => void;
  onFilePreviewPath?: (target: string | FileOperation) => void;
  onOpenSubagent?: (sessionId: string) => void;
  isSubagent?: boolean;
  canCloneMessages?: boolean;
  onCloneMessage?: CloneMessageHandler;
  onCancelCloneSummary?: (operationId: string) => Promise<void>;
  activeSessionTab?: SessionTab | null;
  onCreateCloneGitBranch?: (path: string, cloneSessionId: string) => Promise<string>;
  onLinkCloneGitBranch?: (path: string, cloneSessionId: string, branchName: string) => Promise<void>;
}
