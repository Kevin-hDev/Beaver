import { useCloneTabArchive } from "@/hooks/use-clone-tab-archive";
import { useSessionTabGitSwitch } from "@/hooks/use-session-tab-git-switch";
import type { useGitBranch } from "@/hooks/use-git-branch";
import type { useSessionTabs } from "@/hooks/use-session-tabs";

interface Options {
  rootSessionId: string | null | undefined;
  projectPath?: string;
  git: ReturnType<typeof useGitBranch>;
  sessionTabs: ReturnType<typeof useSessionTabs>;
}

export function useAgentLocalTabGit({
  rootSessionId,
  projectPath,
  git,
  sessionTabs,
}: Options) {
  const tabSwitch = useSessionTabGitSwitch({
    rootSessionId,
    tabs: sessionTabs.tabs,
    git,
    projectPath,
    onSelectTab: sessionTabs.selectTab,
    onUnlinkCloneGitBranch: sessionTabs.unlinkCloneGitBranch,
    onSaveMainCheckpointBranch: sessionTabs.saveMainCheckpointBranch,
  });
  const archive = useCloneTabArchive({
    tabs: sessionTabs.tabs,
    projectPath,
    onCloseTab: sessionTabs.closeTab,
    onCloseTabWithGitCleanup: sessionTabs.closeTabWithGitCleanup,
    getMainBranch: tabSwitch.getMainBranch,
    onAfterCleanup: git.refresh,
  });

  return {
    selectTab: tabSwitch.selectTab,
    closeTab: archive.closeTab,
    dialogs: (
      <>
        {tabSwitch.conflictDialog}
        {archive.dialog}
      </>
    ),
  };
}
