import { useTranslation } from "react-i18next";
import type { SessionSummaryGitState } from "./session-summary-git-types";
import { uncommittedChangeSummary } from "@/lib/git-file-preview";

interface SessionChanges {
  additions: number;
  deletions: number;
}

interface SessionSummaryChangeStatsProps {
  sessionChanges: SessionChanges;
  git?: SessionSummaryGitState;
}

interface DisplayedChanges {
  additions: number | null;
  deletions: number | null;
  state: "ready" | "loading" | "error";
  partialFiles?: number;
}

export function SessionSummaryChangeStats({
  sessionChanges,
  git,
}: SessionSummaryChangeStatsProps) {
  const { t } = useTranslation();
  const changes = resolveDisplayedChanges(sessionChanges, git);
  const title = changes.state === "loading"
    ? t("common.loading")
    : changes.state === "error"
      ? t("agentLocal.sessionSummary.modificationsUnavailable")
      : changes.partialFiles
        ? t("agentLocal.sessionSummary.modificationsPartial", { count: changes.partialFiles })
        : undefined;

  return (
    <span className="ssb-change-stats" title={title} aria-label={title}>
      <span className="ssb-change-add">+{changes.additions ?? "…"}</span>
      <span className="ssb-change-del">-{changes.deletions ?? "…"}</span>
    </span>
  );
}

export function resolveDisplayedChanges(
  sessionChanges: SessionChanges,
  git?: SessionSummaryGitState,
): DisplayedChanges {
  if (!git) return ready(sessionChanges);
  if (git.isLoading && !git.isGitRepo) return unknown("loading");
  if (!git.isGitRepo) return ready(sessionChanges);
  if (git.dirtyCount === 0) return ready({ additions: 0, deletions: 0 });
  if (git.uncommittedSnapshotStatus === "loading") return unknown("loading");
  if (git.uncommittedSnapshotStatus === "error" || !git.uncommittedSnapshot) {
    return unknown("error");
  }

  const changes = uncommittedChangeSummary(git.uncommittedSnapshot);
  return {
    ...changes,
    state: "ready",
    partialFiles: git.uncommittedSnapshot.truncated
      ? git.uncommittedSnapshot.total_files
      : undefined,
  };
}

function ready(changes: SessionChanges): DisplayedChanges {
  return { ...changes, state: "ready" };
}

function unknown(state: "loading" | "error"): DisplayedChanges {
  return { additions: null, deletions: null, state };
}
