import { ChatProjectControls } from "./chat-project-controls";
import { ScrollBottomButton } from "./scroll-bottom-button";
import type { useGitBranch } from "@/hooks/use-git-branch";
import type { useSessionProject } from "@/hooks/use-session-project";
import type { Project } from "@/types/agent";

interface ChatInputFooterProps {
  projects: Project[];
  projectState: ReturnType<typeof useSessionProject>;
  git: ReturnType<typeof useGitBranch>;
  showScrollBottom: boolean;
  centerSlot?: React.ReactNode;
  onScrollBottom: () => void;
  onWorktreeSelect: (path: string, branch: string) => void;
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
  showScrollBottom,
  centerSlot,
  onScrollBottom,
  onWorktreeSelect,
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
          onBranchReady={onBranchReady}
          cloneGitBranch={cloneGitBranch}
        />
      </div>
      {centerSlot && <div className="chat-input-center-slot">{centerSlot}</div>}
      {showScrollBottom && <ScrollBottomButton variant="inline" onClick={onScrollBottom} />}
    </div>
  );
}
