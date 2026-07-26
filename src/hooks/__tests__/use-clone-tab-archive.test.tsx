import { act, render, renderHook, screen, fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useCloneTabArchive } from "../use-clone-tab-archive";
import { showToast } from "@/lib/toast-emitter";
import type { SessionTabs } from "@/types/agent";

vi.mock("@/hooks/use-keyboard", () => ({ useKeyboard: () => {} }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const tabs: SessionTabs = {
  active_tab_id: "branch-1",
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

describe("useCloneTabArchive", () => {
  it("refuse le nettoyage sans checkpoint principal", () => {
    const cleanup = vi.fn();
    const { result } = renderHook(() => useCloneTabArchive({
      tabs,
      projectPath: "/repo",
      onCloseTab: vi.fn(),
      onCloseTabWithGitCleanup: cleanup,
      getMainBranch: () => undefined,
    }));

    act(() => result.current.closeTab("branch-1"));
    render(<>{result.current.dialog}</>);
    fireEvent.click(screen.getByRole("button", { name: "agentLocal.clone.gitArchiveCleanup" }));

    expect(cleanup).not.toHaveBeenCalled();
    expect(showToast).toHaveBeenCalledWith("agentLocal.clone.gitMissingCheckpoint", "error", 3000);
  });

  it("rafraîchit git après un nettoyage réussi", async () => {
    const afterCleanup = vi.fn().mockResolvedValue(undefined);
    const cleanup = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useCloneTabArchive({
      tabs,
      projectPath: "/repo",
      onCloseTab: vi.fn(),
      onCloseTabWithGitCleanup: cleanup,
      getMainBranch: () => "main",
      onAfterCleanup: afterCleanup,
    }));

    act(() => result.current.closeTab("branch-1"));
    render(<>{result.current.dialog}</>);
    fireEvent.click(screen.getByRole("button", { name: "agentLocal.clone.gitArchiveCleanup" }));

    await vi.waitFor(() => expect(afterCleanup).toHaveBeenCalled());
  });

  it("explique qu'une branche principale est protégée", async () => {
    const cleanup = vi.fn().mockRejectedValue({ kind: "protected_branch" });
    const { result } = renderHook(() => useCloneTabArchive({
      tabs,
      projectPath: "/repo",
      onCloseTab: vi.fn(),
      onCloseTabWithGitCleanup: cleanup,
      getMainBranch: () => "main",
    }));

    act(() => result.current.closeTab("branch-1"));
    render(<>{result.current.dialog}</>);
    fireEvent.click(screen.getByRole("button", { name: "agentLocal.clone.gitArchiveCleanup" }));

    await vi.waitFor(() => expect(showToast).toHaveBeenCalledWith(
      "branches.errorProtectedBranch",
      "error",
      3000,
    ));
  });
});
