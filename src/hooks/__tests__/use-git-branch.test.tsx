import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useGitBranch } from "@/hooks/use-git-branch";
import type { GitUncommittedSnapshot } from "@/hooks/git-types";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe("useGitBranch", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("distingue une erreur de statut distant de l'absence de remote", async () => {
    mockGitCommands(true);
    const { result } = renderHook(() => useGitBranch("/repo"));

    await waitFor(() => expect(result.current.remoteStatusError).toBe(true));

    expect(result.current.isGitRepo).toBe(true);
    expect(result.current.hasRemote).toBe(false);
    expect(result.current.remoteStatusError).toBe(true);
  });

  it("arrête le watcher précédent au changement de projet et au démontage", async () => {
    mockGitCommands(false);
    const view = renderHook(
      ({ path }: { path?: string }) => useGitBranch(path),
      { initialProps: { path: "/repo" } },
    );

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("start_git_watcher", { path: "/repo" }));
    view.rerender({ path: "/other" });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("stop_git_watcher", { path: "/repo" });
      expect(invoke).toHaveBeenCalledWith("start_git_watcher", { path: "/other" });
    });

    view.unmount();
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("stop_git_watcher", { path: "/other" });
    });
  });

  it("affiche la branche sans attendre les informations Git secondaires", async () => {
    const worktrees = deferred<never[]>();
    const remote = deferred<RemoteStatus>();
    mockGitCommands(false, { worktrees: worktrees.promise, remote: remote.promise });

    const { result } = renderHook(() => useGitBranch("/repo"));

    await waitFor(() => expect(result.current.isGitRepo).toBe(true), { timeout: 250 });
    expect(result.current.currentBranch).toBe("main");
    expect(result.current.isLoading).toBe(false);
  });

  it("termine un commit sans attendre les informations Git secondaires", async () => {
    mockGitCommands(false);
    const { result } = renderHook(() => useGitBranch("/repo"));
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    const worktrees = deferred<never[]>();
    const remote = deferred<RemoteStatus>();
    mockGitCommands(false, { worktrees: worktrees.promise, remote: remote.promise });

    await act(async () => {
      await expect(result.current.commit("Travail termine")).resolves.toEqual({ ok: true });
    });
  });

  it("charge le snapshot non commit dans le même état Git", async () => {
    mockGitCommands(false);
    const { result } = renderHook(() => useGitBranch("/repo"));

    await waitFor(() => expect(result.current.uncommittedSnapshot).not.toBeNull());

    expect(result.current.uncommittedSnapshot?.head_commit).toBe("a".repeat(40));
    expect(result.current.uncommittedSnapshotStatus).toBe("ready");
  });

  it("distingue le chargement du snapshot d'un worktree propre", async () => {
    const snapshot = deferred<GitUncommittedSnapshot>();
    mockGitCommands(false, { dirtyCount: 1, snapshot: snapshot.promise });
    const { result } = renderHook(() => useGitBranch("/repo"));

    await waitFor(() => {
      expect(result.current.uncommittedSnapshotStatus).toBe("loading");
      expect(result.current.dirtyCount).toBe(1);
    });

    snapshot.resolve(emptySnapshot());
    await waitFor(() => expect(result.current.uncommittedSnapshotStatus).toBe("ready"));
  });

  it("conserve un état d'erreur si le snapshot Git est indisponible", async () => {
    mockGitCommands(false, {
      dirtyCount: 1,
      snapshot: Promise.reject(new Error("unavailable")),
    });
    const { result } = renderHook(() => useGitBranch("/repo"));

    await waitFor(() => expect(result.current.uncommittedSnapshotStatus).toBe("error"));
    expect(result.current.dirtyCount).toBe(1);
    expect(result.current.uncommittedSnapshot).toBeNull();
  });
});

interface RemoteStatus {
  has_remote: boolean;
  is_github: boolean;
  has_remote_branch: boolean;
  ahead: number;
  behind: number;
}

const REMOTE_STATUS: RemoteStatus = {
  has_remote: true,
  is_github: true,
  has_remote_branch: true,
  ahead: 0,
  behind: 0,
};

function mockGitCommands(
  remoteFailure: boolean,
  pending: {
    worktrees?: Promise<never[]>;
    remote?: Promise<RemoteStatus>;
    snapshot?: Promise<GitUncommittedSnapshot>;
    dirtyCount?: number;
  } = {},
) {
  vi.mocked(invoke).mockImplementation((command) => {
    switch (command) {
      case "list_git_branches": return Promise.resolve([]);
      case "get_git_context":
        return Promise.resolve({
          branch: "main",
          is_detached: false,
          dirty_count: pending.dirtyCount ?? 0,
          is_git_repo: true,
        });
      case "list_git_worktrees": return pending.worktrees ?? Promise.resolve([]);
      case "get_git_remote_status":
        if (pending.remote) return pending.remote;
        return remoteFailure
          ? Promise.reject(new Error("unavailable"))
          : Promise.resolve(REMOTE_STATUS);
      case "list_git_uncommitted_files":
        return pending.snapshot ?? Promise.resolve(emptySnapshot());
      case "commit_git_changes":
        return Promise.resolve();
      case "start_git_watcher":
      case "stop_git_watcher":
        return Promise.resolve();
      default:
        return Promise.reject(new Error("unexpected command"));
    }
  });
}

function emptySnapshot(): GitUncommittedSnapshot {
  return {
    head_commit: "a".repeat(40),
    files: [],
    total_files: 0,
    truncated: false,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}
