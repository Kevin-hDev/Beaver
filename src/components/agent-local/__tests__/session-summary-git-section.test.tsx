import { act, fireEvent, render, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SessionSummaryGitSection, type SessionSummaryGitState } from "../session-summary-git-section";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const text: Record<string, string> = {
        "agentLocal.sessionSummary.branch": "Branch",
        "agentLocal.sessionSummary.git.commit": "Commit",
        "agentLocal.sessionSummary.git.push": "Push",
        "agentLocal.sessionSummary.git.commitTitle": "Commit changes",
        "agentLocal.sessionSummary.git.commitDescription": "Describe the work",
        "agentLocal.sessionSummary.git.cancel": "Cancel",
        "agentLocal.sessionSummary.git.confirmCommit": "Commit",
        "agentLocal.sessionSummary.git.pushError": "Unable to send the branch",
        "agentLocal.sessionSummary.git.destinationGithub": "GitHub",
        "agentLocal.sessionSummary.git.remoteStatusUnavailable": "Remote status unavailable",
      };
      if (key.endsWith(".toggle")) return `Branch: ${typeof opts?.branch === "string" ? opts.branch : ""}`;
      if (key.endsWith(".changesToCommit")) return `${numberValue(opts?.count)} files to Commit`;
      if (key.endsWith(".commitsToPush")) {
        return `${numberValue(opts?.count)} commits to Push to GitHub · ${textValue(opts?.branch)}`;
      }
      if (key.endsWith(".localBranch")) return `Local branch · no Push to ${textValue(opts?.branch)}`;
      return text[key] ?? key;
    },
  }),
}));

const githubAuth = vi.hoisted(() => ({
  request: vi.fn(),
  onConnected: undefined as (() => void) | undefined,
}));

vi.mock("@/hooks/use-github-branch-auth", () => ({
  useGithubBranchAuth: (onConnected?: () => void) => {
    githubAuth.onConnected = onConnected;
    return { open: false, state: "idle", request: githubAuth.request };
  },
}));

const baseGit: SessionSummaryGitState = {
  repositoryPath: "/repo",
  isGitRepo: true,
  isLoading: false,
  currentBranch: "main",
  branches: [{ name: "main", is_current: true, is_remote: false, dirty_count: 0 }],
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

describe("SessionSummaryGitSection", () => {
  beforeEach(() => {
    githubAuth.request.mockReset();
    githubAuth.onConnected = undefined;
  });

  it("deplie la branche et enregistre les modifications", async () => {
    const commit = vi.fn().mockResolvedValue({ ok: true });
    const listDirtyFiles = vi.fn().mockResolvedValue([{
      path: "src/app.tsx", status: "modified", additions: 3, deletions: 1,
    }]);
    const { getByRole, queryByRole } = render(
      <SessionSummaryGitSection git={{ ...baseGit, dirtyCount: 1, commit, listDirtyFiles }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    fireEvent.click(getByRole("button", { name: "Commit" }));
    await act(async () => {});
    fireEvent.click(within(getByRole("dialog", { name: "Commit changes" }))
      .getByRole("button", { name: "Commit" }));

    await act(async () => {});
    expect(commit).toHaveBeenCalledWith(undefined);
    expect(queryByRole("dialog", { name: "Commit changes" })).toBeNull();
  });

  it("ferme la fenetre de commit avec Annuler", async () => {
    const { getByRole, queryByRole } = render(
      <SessionSummaryGitSection git={{ ...baseGit, dirtyCount: 1 }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    fireEvent.click(getByRole("button", { name: "Commit" }));
    await act(async () => {});
    fireEvent.click(within(getByRole("dialog", { name: "Commit changes" }))
      .getByRole("button", { name: "Cancel" }));

    expect(queryByRole("dialog", { name: "Commit changes" })).toBeNull();
  });

  it("affiche le push seulement quand des commits sont prets", async () => {
    const push = vi.fn().mockResolvedValue({ ok: true });
    const { getByRole, getByText } = render(
      <SessionSummaryGitSection git={{ ...baseGit, aheadCount: 2, push }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    expect(getByText("2 commits to Push to GitHub · main")).toBeTruthy();
    fireEvent.click(getByRole("button", { name: "Push" }));

    await act(async () => {});
    expect(push).toHaveBeenCalledOnce();
  });

  it("n'ouvre pas GitHub pour une erreur d'authentification d'un autre depot", async () => {
    const push = vi.fn().mockResolvedValue({ ok: false, kind: "authentication_required" });
    const { getByRole, getByText } = render(
      <SessionSummaryGitSection
        git={{ ...baseGit, isGithubRemote: false, aheadCount: 1, push }}
      />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    fireEvent.click(getByRole("button", { name: "Push" }));
    await act(async () => {});

    expect(getByText("agentLocal.sessionSummary.git.authenticationError")).toBeTruthy();
  });

  it("utilise Push quand la branche distante n'est pas encore connue", async () => {
    const push = vi.fn().mockResolvedValue({ ok: true });
    const { getByRole, getByText, queryByText } = render(
      <SessionSummaryGitSection
        git={{ ...baseGit, hasRemoteBranch: false, push }}
      />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    expect(getByText("Local branch · no Push to main")).toBeTruthy();
    expect(queryByText(/Publish/i)).toBeNull();
    fireEvent.click(getByRole("button", { name: "Push" }));

    await act(async () => {});
    expect(push).toHaveBeenCalledOnce();
  });

  it("relance automatiquement le Push apres la connexion GitHub", async () => {
    const push = vi.fn()
      .mockResolvedValueOnce({ ok: false, kind: "authentication_required" })
      .mockResolvedValueOnce({ ok: true });
    const { getByRole } = render(
      <SessionSummaryGitSection git={{ ...baseGit, aheadCount: 1, push }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    fireEvent.click(getByRole("button", { name: "Push" }));
    await waitFor(() => expect(githubAuth.request).toHaveBeenCalledOnce());

    act(() => githubAuth.onConnected?.());
    await waitFor(() => expect(push).toHaveBeenCalledTimes(2));
  });

  it("ne propose pas Push quand la branche distante est plus recente", () => {
    const { getByRole, queryByRole } = render(
      <SessionSummaryGitSection git={{ ...baseGit, aheadCount: 1, behindCount: 1 }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));

    expect(queryByRole("button", { name: "Push" })).toBeNull();
  });

  it("distingue un statut distant indisponible de l'absence de remote", () => {
    const { getByRole, getByText, queryByRole } = render(
      <SessionSummaryGitSection git={{ ...baseGit, remoteStatusError: true }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));

    expect(getByText("Remote status unavailable")).toBeTruthy();
    expect(queryByRole("button", { name: "Push" })).toBeNull();
  });
});

function numberValue(value: unknown) {
  return typeof value === "number" || typeof value === "string" ? value : "";
}

function textValue(value: unknown) {
  return typeof value === "string" ? value : "";
}
