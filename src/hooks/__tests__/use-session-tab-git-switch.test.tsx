import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useSessionTabGitSwitch } from "../use-session-tab-git-switch";
import { showToast } from "@/lib/toast-emitter";
import type { SessionTabs } from "@/types/agent";

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/hooks/use-keyboard", () => ({ useKeyboard: () => {} }));

const tabs: SessionTabs = {
  active_tab_id: "main",
  main_checkpoint_branch: "main",
  tabs: [
    { tab_id: "main", session_id: "root", label: "Main", is_main: true },
    {
      tab_id: "branch-1",
      session_id: "clone",
      label: "Branch 1",
      is_main: false,
      clone_parent_session_id: "root",
      git_branch: "clone-11111111",
    },
  ],
};

function git(overrides = {}) {
  return {
    branches: [{ name: "clone-11111111", is_current: false, is_remote: false, dirty_count: 0 }],
    worktrees: [],
    currentBranch: "main",
    dirtyCount: 0,
    isGitRepo: true,
    isLoading: false,
    refresh: vi.fn(),
    checkout: vi.fn().mockResolvedValue({ ok: true }),
    create: vi.fn(),
    ...overrides,
  };
}

describe("useSessionTabGitSwitch", () => {
  it("checkout la branche liée avant de sélectionner l'onglet clone", async () => {
    const checkout = vi.fn().mockResolvedValue({ ok: true });
    const onSelectTab = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useSessionTabGitSwitch({
      rootSessionId: "root",
      tabs,
      git: git({ checkout }),
      projectPath: "/repo",
      onSelectTab,
      onUnlinkCloneGitBranch: vi.fn(),
      onSaveMainCheckpointBranch: vi.fn(),
    }));

    await result.current.selectTab("branch-1");

    expect(checkout).toHaveBeenCalledWith("clone-11111111");
    await waitFor(() => expect(onSelectTab).toHaveBeenCalledWith("branch-1"));
  });

  it("retire le lien au switch si la branche n'existe plus", async () => {
    const unlink = vi.fn().mockResolvedValue(undefined);
    const onSelectTab = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useSessionTabGitSwitch({
      rootSessionId: "root",
      tabs,
      git: git({ branches: [] }),
      projectPath: "/repo",
      onSelectTab,
      onUnlinkCloneGitBranch: unlink,
      onSaveMainCheckpointBranch: vi.fn(),
    }));

    await result.current.selectTab("branch-1");

    await waitFor(() => expect(unlink).toHaveBeenCalledWith("clone"));
    await waitFor(() => expect(onSelectTab).toHaveBeenCalledWith("branch-1"));
  });

  it("affiche la raison structurée si le checkout échoue", async () => {
    const checkout = vi.fn().mockResolvedValue({ ok: false, kind: "checkout_failed" });
    const { result } = renderHook(() => useSessionTabGitSwitch({
      rootSessionId: "root",
      tabs,
      git: git({ checkout }),
      projectPath: "/repo",
      onSelectTab: vi.fn(),
      onUnlinkCloneGitBranch: vi.fn(),
      onSaveMainCheckpointBranch: vi.fn(),
    }));

    await result.current.selectTab("branch-1");

    expect(showToast).toHaveBeenCalledWith("branches.errorCheckoutFailed", "error", 3000);
  });

  it("persiste la branche principale quand aucun checkpoint n'existe", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    renderHook(() => useSessionTabGitSwitch({
      rootSessionId: "root",
      tabs: { ...tabs, main_checkpoint_branch: undefined },
      git: git({ currentBranch: "main" }),
      projectPath: "/repo",
      onSelectTab: vi.fn(),
      onUnlinkCloneGitBranch: vi.fn(),
      onSaveMainCheckpointBranch: save,
    }));

    await waitFor(() => expect(save).toHaveBeenCalledWith("main"));
  });

  it("ne remplace pas un checkpoint principal existant", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    renderHook(() => useSessionTabGitSwitch({
      rootSessionId: "root",
      tabs,
      git: git({ currentBranch: "Delete-branch" }),
      projectPath: "/repo",
      onSelectTab: vi.fn(),
      onUnlinkCloneGitBranch: vi.fn(),
      onSaveMainCheckpointBranch: save,
    }));

    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(save).not.toHaveBeenCalled();
  });

  it("ne persiste pas une branche de clone comme branche principale", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    renderHook(() => useSessionTabGitSwitch({
      rootSessionId: "root",
      tabs,
      git: git({ currentBranch: "clone-11111111" }),
      projectPath: "/repo",
      onSelectTab: vi.fn(),
      onUnlinkCloneGitBranch: vi.fn(),
      onSaveMainCheckpointBranch: save,
    }));

    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(save).not.toHaveBeenCalled();
  });
});
