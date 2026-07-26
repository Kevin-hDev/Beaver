import { act, fireEvent, render, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GitMergeDialog } from "../git-merge-dialog";
import { SessionSummaryGitSection, type SessionSummaryGitState } from "../session-summary-git-section";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const text: Record<string, string> = {
        "agentLocal.sessionSummary.branch": "Branch",
        "agentLocal.sessionSummary.git.commit": "Commit",
        "agentLocal.sessionSummary.git.push": "Push",
        "agentLocal.sessionSummary.git.mergeSource": "Source branch",
        "agentLocal.sessionSummary.git.cancel": "Cancel",
        "agentLocal.sessionSummary.git.commitDescription": "Commit message",
      };
      if (key.endsWith(".toggle")) return `Branch: ${stringValue(opts?.branch)}`;
      if (key.endsWith(".merge")) return `Merge into ${stringValue(opts?.branch)}`;
      if (key.endsWith(".mergeTitle")) return `Merge into ${stringValue(opts?.branch)}`;
      if (key.endsWith(".mergeDescription")) return `Choose a branch for ${stringValue(opts?.branch)}`;
      if (key.endsWith(".mergeSummary")) return `${stringValue(opts?.count)} commits into ${stringValue(opts?.branch)}`;
      if (key.endsWith(".mergeDirty")) return `${stringValue(opts?.count)} local changes`;
      if (key.endsWith(".confirmMerge")) return `Merge into ${stringValue(opts?.branch)}`;
      if (key.endsWith(".commitAndMerge")) {
        return `Commit then Merge into ${stringValue(opts?.branch)}`;
      }
      return text[key] ?? key;
    },
  }),
}));

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  GitBranch: () => <span />,
  X: () => <span />,
}));

vi.mock("@/hooks/use-github-branch-auth", () => ({
  useGithubBranchAuth: () => ({ open: false, state: "idle", request: vi.fn() }),
}));

describe("SessionSummaryGitSection Merge", () => {
  it.each(["dark", "light"])("rend la fenêtre compacte avec le thème %s", async (theme) => {
    const { container, findByRole } = render(
      <div data-theme={theme}>
        <GitMergeDialog
          branches={baseGit.branches}
          targetBranch="main"
          busy={false}
          onCancel={vi.fn()}
          onPreview={() => Promise.resolve({
            source_branch: "feature",
            target_branch: "main",
            commits: 1,
            dirty_files: [],
          })}
          onMerge={vi.fn()}
        />
      </div>,
    );

    expect(await findByRole("dialog", { name: "Merge into main" })).toBeTruthy();
    expect(container.querySelector(`[data-theme="${theme}"] .gmd-dialog`)).toBeTruthy();
  });

  it("permet de choisir la branche source puis lance le Merge dans la branche active", async () => {
    const previewBranchMerge = vi.fn((source: string) => Promise.resolve({
      source_branch: source,
      target_branch: "main",
      commits: source === "release" ? 2 : 1,
      dirty_files: [],
    }));
    const mergeBranch = vi.fn().mockResolvedValue({ ok: true });
    const { getByRole, findByRole } = render(
      <SessionSummaryGitSection git={{ ...baseGit, previewBranchMerge, mergeBranch }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    fireEvent.click(getByRole("button", { name: "Merge into main" }));
    const dialog = await findByRole("dialog", { name: "Merge into main" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Source branch" }));
    fireEvent.click(within(dialog).getByRole("option", { name: "release" }));
    await waitFor(() => expect(within(dialog).getByText("2 commits into main")).toBeTruthy());
    fireEvent.click(within(dialog).getByRole("button", { name: "Merge into main" }));

    await waitFor(() => expect(mergeBranch).toHaveBeenCalledWith("release", "main", false, undefined));
  });

  it("propose Commit puis Merge lorsque la branche active contient des modifications", async () => {
    const mergeBranch = vi.fn().mockResolvedValue({ ok: true });
    const previewBranchMerge = vi.fn().mockResolvedValue({
      source_branch: "feature",
      target_branch: "main",
      commits: 1,
      dirty_files: [{ path: "src/app.tsx", status: "modified", additions: 2, deletions: 1 }],
    });
    const { getByRole, findByRole } = render(
      <SessionSummaryGitSection git={{
        ...baseGit,
        dirtyCount: 1,
        previewBranchMerge,
        mergeBranch,
      }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    fireEvent.click(getByRole("button", { name: "Merge into main" }));
    const dialog = await findByRole("dialog", { name: "Merge into main" });
    await waitFor(() => expect(within(dialog).getByText("src/app.tsx")).toBeTruthy());
    fireEvent.change(within(dialog).getByRole("textbox"), { target: { value: "Prepare merge" } });
    fireEvent.click(within(dialog).getByRole("button", { name: "Commit then Merge into main" }));

    await act(async () => {});
    expect(mergeBranch).toHaveBeenCalledWith("feature", "main", true, "Prepare merge");
  });

  it("ne propose pas Merge quand HEAD est détaché", () => {
    const { getByRole, queryByRole } = render(
      <SessionSummaryGitSection git={{
        ...baseGit,
        currentBranch: "HEAD",
        branches: baseGit.branches.map((branch) => ({ ...branch, is_current: false })),
      }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: HEAD" }));

    expect(queryByRole("button", { name: /Merge into/ })).toBeNull();
  });

  it("affiche toujours la branche de destination sur le bouton", () => {
    const branches = baseGit.branches.map((branch) => ({
      ...branch,
      is_current: branch.name === "feature",
    }));
    const { getByRole } = render(
      <SessionSummaryGitSection git={{ ...baseGit, branches, currentBranch: "feature" }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: feature" }));

    expect(getByRole("button", { name: "Merge into feature" })).toBeTruthy();
  });

  it("remplace Push par Commit après un nouveau changement", () => {
    const { getByRole, queryByRole, rerender } = render(
      <SessionSummaryGitSection git={{ ...baseGit, aheadCount: 1 }} />,
    );
    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    expect(getByRole("button", { name: "Push" })).toBeTruthy();

    rerender(<SessionSummaryGitSection git={{ ...baseGit, aheadCount: 1, dirtyCount: 1 }} />);

    expect(queryByRole("button", { name: "Push" })).toBeNull();
    expect(getByRole("button", { name: "Commit" })).toBeTruthy();
    expect(getByRole("button", { name: "Merge into main" })).toBeTruthy();
  });
});

const baseGit: SessionSummaryGitState = {
  repositoryPath: "/repo",
  isGitRepo: true,
  isLoading: false,
  currentBranch: "main",
  branches: [
    { name: "main", is_current: true, is_remote: false, dirty_count: 0 },
    { name: "feature", is_current: false, is_remote: false, dirty_count: 0 },
    { name: "release", is_current: false, is_remote: false, dirty_count: 0 },
  ],
  dirtyCount: 0,
  hasRemote: true,
  remoteStatusError: false,
  isGithubRemote: true,
  hasRemoteBranch: true,
  aheadCount: 0,
  behindCount: 0,
  uncommittedSnapshotStatus: "ready",
  worktrees: [],
  listDirtyFiles: vi.fn().mockResolvedValue([]),
  listCommits: vi.fn().mockResolvedValue({ commits: [] }),
  listCommitFiles: vi.fn().mockResolvedValue([]),
  commit: vi.fn().mockResolvedValue({ ok: true }),
  push: vi.fn().mockResolvedValue({ ok: true }),
  previewBranchMerge: vi.fn(),
  mergeBranch: vi.fn().mockResolvedValue({ ok: true }),
  refresh: vi.fn(),
};

function stringValue(value: unknown) {
  return typeof value === "string" || typeof value === "number" ? String(value) : "";
}
