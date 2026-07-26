import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, GitBranch } from "@/components/ui/icons";
import { SessionSummaryGitActions } from "./session-summary-git-actions";
import type { SessionSummaryGitState } from "./session-summary-git-types";

export type { SessionSummaryGitState } from "./session-summary-git-types";

interface SessionSummaryGitSectionProps {
  git?: SessionSummaryGitState;
}

export function SessionSummaryGitSection({ git }: SessionSummaryGitSectionProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const branch = branchLabel(git, t);

  if (!git?.isGitRepo) {
    return <StaticBranchRow branch={branch} />;
  }

  return (
    <>
      <button
        className="ssb-row ssb-git-toggle"
        type="button"
        aria-label={t("agentLocal.sessionSummary.git.toggle", { branch })}
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <GitBranch size="var(--icon-md)" className="ssb-row-icon" />
        <span className="ssb-row-label">{t("agentLocal.sessionSummary.branch")}</span>
        <span className="ssb-row-value" title={branch}>{branch}</span>
        <CaretDown className={`ssb-section-caret ${open ? "ssb-section-caret-open" : ""}`} size="var(--icon-sm)" />
      </button>
      <div className={`ssb-accordion ${open ? "ssb-accordion-open" : ""}`}>
        <div className="ssb-accordion-inner">
          <div className="ssb-git-panel">
            <div className="ssb-git-status">{statusLabel(git, t)}</div>
            <SessionSummaryGitActions git={git} />
          </div>
        </div>
      </div>
    </>
  );
}

function StaticBranchRow({ branch }: { branch: string }) {
  const { t } = useTranslation();
  return (
    <div className="ssb-row">
      <GitBranch size="var(--icon-md)" className="ssb-row-icon" />
      <span className="ssb-row-label">{t("agentLocal.sessionSummary.branch")}</span>
      <span className="ssb-row-value" title={branch}>{branch}</span>
    </div>
  );
}

function branchLabel(git: SessionSummaryGitState | undefined, t: (key: string) => string) {
  if (!git) return t("agentLocal.sessionSummary.noGit");
  if (git.isLoading && !git.currentBranch) return t("common.loading");
  if (!git.isGitRepo) return t("agentLocal.sessionSummary.noGit");
  return git.currentBranch || t("branches.detachedHead");
}

function statusLabel(git: SessionSummaryGitState, t: (key: string, values?: Record<string, unknown>) => string) {
  if (git.dirtyCount > 0) return t("agentLocal.sessionSummary.git.changesToCommit", { count: git.dirtyCount });
  if (git.remoteStatusError) return t("agentLocal.sessionSummary.git.remoteStatusUnavailable");
  if (!git.hasRemote) return t("agentLocal.sessionSummary.git.noRemote");
  if (git.behindCount > 0) return t("agentLocal.sessionSummary.git.remoteChanges");
  if (git.aheadCount > 0) {
    return t("agentLocal.sessionSummary.git.commitsToPush", {
      count: git.aheadCount,
      branch: git.currentBranch,
      destination: t(git.isGithubRemote
        ? "agentLocal.sessionSummary.git.destinationGithub"
        : "agentLocal.sessionSummary.git.destinationRemote"),
    });
  }
  if (!git.hasRemoteBranch) {
    return t("agentLocal.sessionSummary.git.localBranch", { branch: git.currentBranch });
  }
  return t("agentLocal.sessionSummary.git.upToDate");
}
