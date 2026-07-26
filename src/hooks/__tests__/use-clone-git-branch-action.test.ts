import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useCloneGitBranchAction } from "../use-clone-git-branch-action";
import type { SessionTab } from "@/types/agent";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));

const cloneTab: SessionTab = {
  tab_id: "branch-1",
  session_id: "clone",
  label: "Branch 1",
  is_main: false,
  clone_parent_session_id: "root",
};

function git(overrides = {}) {
  return {
    branches: [],
    worktrees: [],
    currentBranch: "main",
    dirtyCount: 0,
    isGitRepo: true,
    isLoading: false,
    refresh: vi.fn().mockResolvedValue(undefined),
    checkout: vi.fn(),
    create: vi.fn(),
    ...overrides,
  };
}

describe("useCloneGitBranchAction", () => {
  beforeEach(() => vi.clearAllMocks());

  it("n'affiche le bouton que pour un clone git non lié", () => {
    const { result, rerender } = renderHook(
      ({ tab }) => useCloneGitBranchAction({
        projectPath: "/repo",
        git: git(),
        isStreaming: false,
        activeSessionTab: tab,
        onCreateCloneGitBranch: vi.fn(),
      }),
      { initialProps: { tab: cloneTab } },
    );
    // Clone sans branche liée → bouton visible, BranchSelector non locké.
    expect(result.current.visible).toBe(true);
    expect(result.current.state).toBe("idle");

    // Clone avec branche liée → bouton masqué (le BranchSelector affiche la branche).
    rerender({ tab: { ...cloneTab, git_branch: "clone-11111111" } });
    expect(result.current.visible).toBe(false);
  });

  it("passe en succès après création", async () => {
    const create = vi.fn().mockResolvedValue("clone-11111111");
    const { result } = renderHook(() => useCloneGitBranchAction({
      projectPath: "/repo",
      git: git(),
      isStreaming: false,
      activeSessionTab: cloneTab,
      onCreateCloneGitBranch: create,
    }));

    act(() => result.current.onCreate());

    await waitFor(() => expect(create).toHaveBeenCalledWith("/repo", "clone"));
    await waitFor(() => expect(result.current.state).toBe("success"));
    expect(result.current.label).toBe("agentLocal.clone.gitCreated");
  });

  it("désactive la création pendant le stream", () => {
    const create = vi.fn();
    const { result } = renderHook(() => useCloneGitBranchAction({
      projectPath: "/repo",
      git: git(),
      isStreaming: true,
      activeSessionTab: cloneTab,
      onCreateCloneGitBranch: create,
    }));

    expect(result.current.visible).toBe(true);
    expect(result.current.disabled).toBe(true);
    act(() => result.current.onCreate());
    expect(create).not.toHaveBeenCalled();
  });
});
