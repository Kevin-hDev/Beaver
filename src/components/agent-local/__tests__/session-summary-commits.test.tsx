import { act, fireEvent, render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SessionSummaryCommits } from "../session-summary-commits";
import type { GitCommitFile, GitCommitSummary } from "@/hooks/git-types";
import type { SessionSummaryGitState } from "../session-summary-git-types";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "fr" },
    t: (key: string) => ({
      "agentLocal.sessionSummary.commits.toggle": "Afficher les commits",
      "agentLocal.sessionSummary.commits.title": "Commits",
      "agentLocal.sessionSummary.commits.empty": "Aucun commit",
      "agentLocal.sessionSummary.commits.error": "Erreur",
      "agentLocal.sessionSummary.commits.noMessage": "Sans message",
      "agentLocal.sessionSummary.commits.noFiles": "Aucun fichier",
      "common.loading": "Chargement",
    } as Record<string, string>)[key] ?? key,
  }),
}));

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretLeft: () => <span />,
}));

vi.mock("@/components/file-preview/file-icon", () => ({
  FileIcon: () => <span />,
}));

describe("SessionSummaryCommits", () => {
  it("ouvre un commit puis transmet le fichier choisi", async () => {
    const commit = {
      id: "a".repeat(40),
      short_id: "aaaaaaaa",
      message: "Premier commit",
      timestamp: 1_700_000_000,
    };
    const file = {
      path: "src/app.ts",
      status: "modified" as const,
      additions: 3,
      deletions: 1,
    };
    const git = gitState({
      listCommits: vi.fn().mockResolvedValue({ commits: [commit] }),
      listCommitFiles: vi.fn().mockResolvedValue([file]),
    });
    const onOpenFile = vi.fn();
    const view = render(<SessionSummaryCommits git={git} onOpenFile={onOpenFile} />);

    fireEvent.click(view.getByRole("button", { name: "Afficher les commits" }));
    await waitFor(() => expect(view.getByText("Premier commit")).toBeTruthy());
    fireEvent.click(view.getByRole("button", { name: /Premier commit/ }));
    await waitFor(() => expect(view.getByText("src/app.ts")).toBeTruthy());
    fireEvent.click(view.getByRole("button", { name: /src\/app.ts/ }));

    expect(onOpenFile).toHaveBeenCalledWith(commit, file);
  });

  it("ignore les fichiers d'un ancien commit après une nouvelle sélection", async () => {
    const first = commit("a", "Premier commit");
    const second = commit("b", "Deuxième commit");
    const firstRequest = deferred<GitCommitFile[]>();
    const secondRequest = deferred<GitCommitFile[]>();
    const git = gitState({
      listCommits: vi.fn().mockResolvedValue({ commits: [first, second] }),
      listCommitFiles: vi.fn((id: string) => id === first.id ? firstRequest.promise : secondRequest.promise),
    });
    const view = render(<SessionSummaryCommits git={git} />);

    fireEvent.click(view.getByRole("button", { name: "Afficher les commits" }));
    await waitFor(() => expect(view.getByText("Premier commit")).toBeTruthy());
    fireEvent.click(view.getByRole("button", { name: /Premier commit/ }));
    fireEvent.click(view.getByRole("button", { name: /Premier commit/ }));
    fireEvent.click(view.getByRole("button", { name: /Deuxième commit/ }));

    await act(async () => {
      secondRequest.resolve([file("second.ts")]);
      await secondRequest.promise;
    });
    await waitFor(() => expect(view.getByText("second.ts")).toBeTruthy());
    await act(async () => {
      firstRequest.resolve([file("first.ts")]);
      await firstRequest.promise;
    });

    expect(view.queryByText("first.ts")).toBeNull();
    expect(view.getByText("second.ts")).toBeTruthy();
  });

  it("charge la page suivante en approchant de la fin", async () => {
    const first = commit("a", "Premier commit");
    const second = commit("b", "Deuxième commit");
    const listCommits = vi.fn()
      .mockResolvedValueOnce({ commits: [first], next_cursor: first.id })
      .mockResolvedValueOnce({ commits: [second] });
    const view = render(<SessionSummaryCommits git={gitState({ listCommits })} />);

    fireEvent.click(view.getByRole("button", { name: "Afficher les commits" }));
    await waitFor(() => expect(view.getByText("Premier commit")).toBeTruthy());
    const scroll = view.container.querySelector(".ssbc-scroll") as HTMLDivElement;
    Object.defineProperties(scroll, {
      scrollHeight: { value: 100 },
      scrollTop: { value: 70 },
      clientHeight: { value: 20 },
    });
    fireEvent.scroll(scroll);

    await waitFor(() => expect(view.getByText("Deuxième commit")).toBeTruthy());
    expect(listCommits).toHaveBeenLastCalledWith(first.id);
  });
});

function commit(seed: string, message: string): GitCommitSummary {
  return { id: seed.repeat(40), short_id: seed.repeat(8), message, timestamp: 1_700_000_000 };
}

function file(path: string): GitCommitFile {
  return { path, status: "modified", additions: 1, deletions: 1 };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

function gitState(overrides: Partial<SessionSummaryGitState>): SessionSummaryGitState {
  return {
    repositoryPath: "/repo",
    isGitRepo: true,
    isLoading: false,
    currentBranch: "main",
    branches: [],
    worktrees: [],
    dirtyCount: 0,
    hasRemote: false,
    remoteStatusError: false,
    isGithubRemote: false,
    hasRemoteBranch: false,
    aheadCount: 0,
    behindCount: 0,
    uncommittedSnapshotStatus: "ready",
    listDirtyFiles: vi.fn().mockResolvedValue([]),
    listCommits: vi.fn().mockResolvedValue({ commits: [] }),
    listCommitFiles: vi.fn().mockResolvedValue([]),
    commit: vi.fn().mockResolvedValue({ ok: true }),
    push: vi.fn().mockResolvedValue({ ok: true }),
    previewBranchMerge: vi.fn(),
    mergeBranch: vi.fn().mockResolvedValue({ ok: true }),
    refresh: vi.fn(),
    ...overrides,
  };
}
