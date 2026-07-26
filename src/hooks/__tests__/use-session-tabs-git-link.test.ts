import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useSessionTabs } from "../use-session-tabs";
import type { SessionTabs } from "@/types/agent";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

vi.mock("../use-session-activity-indicators", () => ({
  clearSessionRunning: vi.fn(),
  markSessionComplete: vi.fn(),
  markSessionRunning: vi.fn(),
}));

const cloneTabs: SessionTabs = {
  active_tab_id: "branch-1",
  tabs: [
    { tab_id: "main", session_id: "root", label: "Main", is_main: true },
    { tab_id: "branch-1", session_id: "clone", label: "Branche 1", is_main: false },
  ],
};

describe("useSessionTabs git branch link", () => {
  beforeEach(() => vi.clearAllMocks());

  it("lie une branche git manuelle au clone actif", async () => {
    const linkedTabs: SessionTabs = {
      ...cloneTabs,
      tabs: cloneTabs.tabs.map((tab) =>
        tab.session_id === "clone" ? { ...tab, git_branch: "feature/manual" } : tab),
    };
    vi.mocked(invoke).mockImplementation((command: string) => {
      if (command === "list_session_tabs") return Promise.resolve(cloneTabs);
      if (command === "link_clone_git_branch") return Promise.resolve(linkedTabs);
      return Promise.resolve(cloneTabs);
    });
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(cloneTabs));

    await act(async () => {
      await result.current.linkCloneGitBranch("/repo", "clone", "feature/manual");
    });

    expect(invoke).toHaveBeenCalledWith("link_clone_git_branch", {
      sessionId: "root",
      cloneSessionId: "clone",
      path: "/repo",
      branchName: "feature/manual",
    });
    await waitFor(() => expect(result.current.tabs).toEqual(linkedTabs));
  });
});
