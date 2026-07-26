import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { ProjectSelector } from "./project-selector";
import "./chat-project-controls.css";
import { BranchSelector } from "./branch-selector";
import { BranchConflictDialog } from "./branch-conflict-dialog";
import { BranchGithubAuthDialog } from "./branch-github-auth-dialog";
import { CloneGitBranchButton } from "./clone-git-branch-button";
import { useGithubBranchAuth } from "@/hooks/use-github-branch-auth";
import type { useGitBranch } from "@/hooks/use-git-branch";
import type { useSessionProject } from "@/hooks/use-session-project";
import { appErrorMessage } from "@/lib/app-error";
import type { Project } from "@/types/agent";

interface ChatProjectControlsProps {
  projects: Project[];
  projectState: ReturnType<typeof useSessionProject>;
  git: ReturnType<typeof useGitBranch>;
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

export function ChatProjectControls({
  projects,
  projectState,
  git,
  onWorktreeSelect,
  onBranchReady,
  cloneGitBranch,
}: ChatProjectControlsProps) {
  const { t } = useTranslation();
  const githubAuth = useGithubBranchAuth(() => void git.refresh());
  const [branchConflict, setBranchConflict] = useState<{
    branch: string;
    dirtyCount: number;
    busy?: boolean;
    error?: string;
  } | null>(null);

  return (
    <>
      <div className="cpc-row">
        <ProjectSelector
          projects={projects}
          selectedProjectId={projectState.selectedProjectId}
          locked={projectState.locked}
          hidden={projectState.hidden}
          onSelect={projectState.setSelectedProjectId}
          onAddProject={() => void projectState.handleAddProject()}
        />
        <BranchSelector
          git={git}
          locked={false}
          onConflict={(branch, dirtyCount) => setBranchConflict({ branch, dirtyCount })}
          onWorktreeSelect={onWorktreeSelect}
          onGithubAuthRequired={githubAuth.request}
          onBranchReady={onBranchReady}
        />
        {cloneGitBranch?.visible && (
          <CloneGitBranchButton
            state={cloneGitBranch.state}
            label={cloneGitBranch.label}
            disabled={cloneGitBranch.disabled}
            onClick={cloneGitBranch.onCreate}
          />
        )}
      </div>

      {branchConflict && projectState.selectedProject && (
        <BranchConflictDialog
          targetBranch={branchConflict.branch}
          dirtyCount={branchConflict.dirtyCount}
          projectPath={projectState.selectedProject.path}
          busy={branchConflict.busy}
          error={branchConflict.error}
          onCancel={() => setBranchConflict(null)}
          onCommitAndSwitch={(branch, commitDescription) => {
            void (async () => {
              setBranchConflict((current) => current ? { ...current, busy: true, error: undefined } : current);
              try {
                await invoke("commit_and_checkout_git_branch", {
                  path: projectState.selectedProject!.path,
                  branchName: branch,
                  commitDescription,
                });
                await git.refresh();
                await onBranchReady?.(branch);
                setBranchConflict(null);
              } catch (error) {
                setBranchConflict((current) => current ? {
                  ...current,
                  busy: false,
                  error: appErrorMessage(error, t, "branches.commitSwitchError"),
                } : current);
                return;
              }
            })();
          }}
        />
      )}
      {githubAuth.open && (
        <BranchGithubAuthDialog
          state={githubAuth.state}
          onCancel={githubAuth.cancel}
          onConnect={() => void githubAuth.connect()}
        />
      )}
    </>
  );
}
