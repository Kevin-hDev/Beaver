import { ChatProjectControls } from "./chat-project-controls";
import type { useGitBranch } from "@/hooks/use-git-branch";
import type { useSessionProject } from "@/hooks/use-session-project";
import type { DirectoryAccessPromptProps } from "./directory-access-prompt";
import type { Project } from "@/types/agent";

interface ChatInputFooterProps {
  projects: Project[];
  projectState: ReturnType<typeof useSessionProject>;
  git: ReturnType<typeof useGitBranch>;
  centerSlot?: React.ReactNode;
  onWorktreeSelect: (path: string, branch: string) => void;
  directoryAccessPrompt?: DirectoryAccessPromptProps;
  onBranchReady?: (branchName: string) => Promise<void> | void;
  cloneGitBranch?: {
    visible: boolean;
    state: "idle" | "loading" | "success";
    label: string;
    disabled?: boolean;
    onCreate: () => void;
  };
}

export function ChatInputFooter({
  projects,
  projectState,
  git,
  centerSlot,
  onWorktreeSelect,
  directoryAccessPrompt,
  onBranchReady,
  cloneGitBranch,
}: ChatInputFooterProps) {
  return (
    <div className="chat-input-under-row">
      <div className="chat-input-project-slot">
        <ChatProjectControls
          projects={projects}
          projectState={projectState}
          git={git}
          onWorktreeSelect={onWorktreeSelect}
          directoryAccessPrompt={directoryAccessPrompt}
          onBranchReady={onBranchReady}
          cloneGitBranch={cloneGitBranch}
        />
      </div>
      {centerSlot && <div className="chat-input-center-slot">{centerSlot}</div>}
    </div>
  );
}
