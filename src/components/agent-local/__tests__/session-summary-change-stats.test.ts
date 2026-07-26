import { describe, expect, it, vi } from "vitest";
import { resolveDisplayedChanges } from "../session-summary-change-stats";
import type { SessionSummaryGitState } from "../session-summary-git-types";

describe("resolveDisplayedChanges", () => {
  it("ne transforme pas un snapshot en chargement en faux zéro", () => {
    const result = resolveDisplayedChanges(
      { additions: 9, deletions: 4 },
      gitState({ dirtyCount: 1, uncommittedSnapshotStatus: "loading" }),
    );

    expect(result).toEqual({ additions: null, deletions: null, state: "loading" });
  });

  it("ne transforme pas une erreur Git en faux zéro", () => {
    const result = resolveDisplayedChanges(
      { additions: 9, deletions: 4 },
      gitState({ dirtyCount: 1, uncommittedSnapshotStatus: "error" }),
    );

    expect(result).toEqual({ additions: null, deletions: null, state: "error" });
  });

  it("signale les totaux partiels lorsque la liste est tronquée", () => {
    const result = resolveDisplayedChanges(
      { additions: 0, deletions: 0 },
      gitState({
        dirtyCount: 201,
        uncommittedSnapshotStatus: "ready",
        uncommittedSnapshot: {
          head_commit: "a".repeat(40),
          files: [{ path: "src/app.ts", status: "modified", additions: 14, deletions: 6 }],
          total_files: 201,
          truncated: true,
        },
      }),
    );

    expect(result).toEqual({
      additions: 14,
      deletions: 6,
      state: "ready",
      partialFiles: 201,
    });
  });
});

function gitState(overrides: Partial<SessionSummaryGitState>): SessionSummaryGitState {
  return {
    repositoryPath: "/repo",
    isGitRepo: true,
    isLoading: false,
    currentBranch: "main",
    branches: [],
    dirtyCount: 0,
    hasRemote: false,
    remoteStatusError: false,
    isGithubRemote: false,
    hasRemoteBranch: false,
    aheadCount: 0,
    behindCount: 0,
    uncommittedSnapshot: null,
    uncommittedSnapshotStatus: "ready",
    worktrees: [],
    listDirtyFiles: vi.fn(),
    listCommits: vi.fn(),
    listCommitFiles: vi.fn(),
    commit: vi.fn(),
    push: vi.fn(),
    previewBranchMerge: vi.fn(),
    mergeBranch: vi.fn(),
    refresh: vi.fn(),
    ...overrides,
  };
}
