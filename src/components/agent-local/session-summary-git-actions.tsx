import { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useGithubBranchAuth } from "@/hooks/use-github-branch-auth";
import type { GitPushTarget } from "@/hooks/git-types";
import { appErrorMessage } from "@/lib/app-error";
import { BranchGithubAuthDialog } from "./branch-github-auth-dialog";
import { GitCommitDialog } from "./git-commit-dialog";
import { GitMergeDialog } from "./git-merge-dialog";
import type { SessionSummaryGitState } from "./session-summary-git-types";

export function SessionSummaryGitActions({ git }: { git: SessionSummaryGitState }) {
  const { t } = useTranslation();
  const [commitOpen, setCommitOpen] = useState(false);
  const [mergeOpen, setMergeOpen] = useState(false);
  const [files, setFiles] = useState<Awaited<ReturnType<typeof git.listDirtyFiles>>>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const pendingPushTarget = useRef<GitPushTarget | undefined>(undefined);
  const retryPush = useCallback(async () => {
    const target = pendingPushTarget.current;
    pendingPushTarget.current = undefined;
    if (!target) return;
    setBusy(true);
    setError(undefined);
    const result = await git.push(target);
    if (!result.ok) setError(appErrorMessage(result, t, "agentLocal.sessionSummary.git.pushError"));
    setBusy(false);
  }, [git, t]);
  const githubAuth = useGithubBranchAuth(() => void retryPush());
  const hasCurrentLocalBranch = git.branches.some(
    (branch) => branch.is_current && branch.name === git.currentBranch,
  );
  const mergeCandidates = hasCurrentLocalBranch
    ? git.branches.filter((branch) => !branch.is_remote && branch.name !== git.currentBranch)
    : [];
  const canPush = git.dirtyCount === 0
    && !git.remoteStatusError
    && git.hasRemote
    && git.behindCount === 0
    && (!git.hasRemoteBranch || git.aheadCount > 0);

  const openCommit = async () => {
    setFiles(await git.listDirtyFiles());
    setError(undefined);
    setMergeOpen(false);
    setCommitOpen(true);
  };
  const commit = async (description?: string) => {
    if (busy) return;
    setBusy(true);
    const result = await git.commit(description);
    if (result.ok) setCommitOpen(false);
    else setError(appErrorMessage(result, t, "agentLocal.sessionSummary.git.commitError"));
    setBusy(false);
  };
  const push = async () => {
    if (busy) return;
    setBusy(true);
    setError(undefined);
    const target = { repositoryPath: git.repositoryPath, branch: git.currentBranch };
    const result = await git.push(target);
    if (!result.ok) {
      if (result.kind === "authentication_required" && git.isGithubRemote) {
        pendingPushTarget.current = target;
        githubAuth.request();
      } else {
        setError(appErrorMessage(result, t, "agentLocal.sessionSummary.git.pushError"));
      }
    }
    setBusy(false);
  };
  const merge = async (
    sourceBranch: string,
    commitChanges: boolean,
    description?: string,
  ) => {
    if (busy) return;
    setBusy(true);
    setError(undefined);
    const result = await git.mergeBranch(
      sourceBranch,
      git.currentBranch,
      commitChanges,
      description,
    );
    if (result.ok) setMergeOpen(false);
    else setError(appErrorMessage(result, t, "agentLocal.sessionSummary.git.mergeError"));
    setBusy(false);
  };

  return (
    <>
      <div className="ssb-git-actions">
        {git.dirtyCount > 0 && (
          <button className="ssb-git-action" type="button" onClick={() => void openCommit()} disabled={busy}>
            {t("agentLocal.sessionSummary.git.commit")}
          </button>
        )}
        {canPush && (
          <button className="ssb-git-action" type="button" onClick={() => void push()} disabled={busy}>
            {t("agentLocal.sessionSummary.git.push")}
          </button>
        )}
        {mergeCandidates.length > 0 && (
          <button className="ssb-git-action" type="button" onClick={() => {
            setError(undefined);
            setCommitOpen(false);
            setMergeOpen(true);
          }} disabled={busy}>
            {t("agentLocal.sessionSummary.git.merge", { branch: git.currentBranch })}
          </button>
        )}
      </div>
      {error && !commitOpen && !mergeOpen && <div className="ssb-git-error">{error}</div>}
      {commitOpen && (
        <GitCommitDialog
          files={files}
          busy={busy}
          error={error}
          onCancel={() => setCommitOpen(false)}
          onCommit={(description) => void commit(description)}
        />
      )}
      {mergeOpen && (
        <GitMergeDialog
          branches={git.branches}
          targetBranch={git.currentBranch}
          busy={busy}
          error={error}
          onCancel={() => setMergeOpen(false)}
          onPreview={(source) => git.previewBranchMerge(source, git.currentBranch)}
          onMerge={(source, commitChanges, description) => {
            void merge(source, commitChanges, description);
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
