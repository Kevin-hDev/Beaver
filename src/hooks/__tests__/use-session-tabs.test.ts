import { act, fireEvent, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useSessionTabs } from "../use-session-tabs";
import type { CloneSessionResult, SessionTabs } from "@/types/agent";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../use-session-activity-indicators", () => ({
  clearSessionRunning: vi.fn(),
  markSessionComplete: vi.fn(),
  markSessionRunning: vi.fn(),
}));

const rootTabs: SessionTabs = {
  active_tab_id: "main",
  tabs: [{ tab_id: "main", session_id: "root", label: "Main", is_main: true }],
};

const cloneTabs: SessionTabs = {
  active_tab_id: "branch-1",
  tabs: [
    ...rootTabs.tabs,
    { tab_id: "branch-1", session_id: "clone", label: "Branche 1", is_main: false },
  ],
};

const cloneResult: CloneSessionResult = {
  root_session_id: "root",
  clone_session_id: "clone",
  operation_id: "op-1",
  tabs: cloneTabs,
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => { resolve = complete; });
  return { promise, resolve };
}

describe("useSessionTabs", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation((command: string, args?: unknown) => {
      if (command === "list_session_tabs") return Promise.resolve(rootTabs);
      if (command === "clone_agent_session") return Promise.resolve(cloneResult);
      if (command === "save_session_tabs") {
        return Promise.resolve((args as { tabs: SessionTabs }).tabs);
      }
      return Promise.resolve(rootTabs);
    });
  });

  it("n'expose jamais les onglets de l'ancienne racine pendant le changement de session", async () => {
    const nextTabs = deferred<SessionTabs>();
    vi.mocked(invoke).mockImplementation((command: string, args?: unknown) => {
      if (command !== "list_session_tabs") return Promise.resolve(rootTabs);
      const sessionId = (args as { sessionId: string }).sessionId;
      return sessionId === "root" ? Promise.resolve(cloneTabs) : nextTabs.promise;
    });

    const { result, rerender } = renderHook(
      ({ sessionId }) => useSessionTabs(sessionId),
      { initialProps: { sessionId: "root" } },
    );
    await waitFor(() => expect(result.current.activeSessionId).toBe("clone"));

    rerender({ sessionId: "next-root" });

    expect(result.current.tabs).toBeNull();
    expect(result.current.activeSessionId).toBe("next-root");

    await act(async () => {
      nextTabs.resolve({
        active_tab_id: "main",
        tabs: [{ tab_id: "main", session_id: "next-root", label: "Main", is_main: true }],
      });
      await nextTabs.promise;
    });
    await waitFor(() => expect(result.current.tabs?.tabs[0]?.session_id).toBe("next-root"));
  });

  it("ignore une sauvegarde de l'ancienne racine terminée après le chargement de la nouvelle", async () => {
    const staleSave = deferred<SessionTabs>();
    const nextRootTabs: SessionTabs = {
      active_tab_id: "main",
      tabs: [{ tab_id: "main", session_id: "next-root", label: "Main", is_main: true }],
    };
    vi.mocked(invoke).mockImplementation((command: string, args?: unknown) => {
      if (command === "list_session_tabs") {
        const sessionId = (args as { sessionId: string }).sessionId;
        return Promise.resolve(sessionId === "root" ? cloneTabs : nextRootTabs);
      }
      if (command === "save_session_tabs") return staleSave.promise;
      return Promise.resolve(rootTabs);
    });

    const { result, rerender } = renderHook(
      ({ sessionId }) => useSessionTabs(sessionId),
      { initialProps: { sessionId: "root" } },
    );
    await waitFor(() => expect(result.current.tabs).toEqual(cloneTabs));

    let pendingSelection!: Promise<void>;
    act(() => {
      pendingSelection = result.current.selectTab("main");
    });
    rerender({ sessionId: "next-root" });
    await waitFor(() => expect(result.current.tabs).toEqual(nextRootTabs));

    await act(async () => {
      staleSave.resolve({ ...cloneTabs, active_tab_id: "main" });
      await pendingSelection;
    });

    expect(result.current.tabs).toEqual(nextRootTabs);
  });

  it("ignore une lecture de l'ancienne racine résolue après celle de la nouvelle", async () => {
    const oldRequest = deferred<SessionTabs>();
    const nextRequest = deferred<SessionTabs>();
    vi.mocked(invoke).mockImplementation((command: string, args?: unknown) => {
      if (command !== "list_session_tabs") return Promise.resolve(rootTabs);
      const sessionId = (args as { sessionId: string }).sessionId;
      return sessionId === "root" ? oldRequest.promise : nextRequest.promise;
    });

    const { result, rerender } = renderHook(
      ({ sessionId }) => useSessionTabs(sessionId),
      { initialProps: { sessionId: "root" } },
    );
    rerender({ sessionId: "next-root" });

    const nextRootTabs: SessionTabs = {
      active_tab_id: "main",
      tabs: [{ tab_id: "main", session_id: "next-root", label: "Main", is_main: true }],
    };
    await act(async () => {
      nextRequest.resolve(nextRootTabs);
      await nextRequest.promise;
    });
    await waitFor(() => expect(result.current.tabs).toEqual(nextRootTabs));

    await act(async () => {
      oldRequest.resolve(cloneTabs);
      await oldRequest.promise;
    });

    expect(result.current.tabs).toEqual(nextRootTabs);
  });

  it("garde l'onglet actif précédent quand le résumé finit en arrière-plan", async () => {
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(rootTabs));

    await act(async () => {
      await result.current.cloneMessage({
        messageId: "m1",
        mode: "summary",
        operationId: "op-frontend",
        shouldActivateOnComplete: () => false,
      });
    });

    expect(invoke).toHaveBeenCalledWith("clone_agent_session", {
      sessionId: "root",
      messageId: "m1",
      mode: "summary",
      customFocus: null,
      operationId: "op-frontend",
    });
    expect(invoke).toHaveBeenCalledWith("save_session_tabs", {
      sessionId: "root",
      tabs: { ...cloneTabs, active_tab_id: "main" },
    });
    expect(result.current.tabs?.active_tab_id).toBe("main");
    expect(result.current.attentionTabIds.has("branch-1")).toBe(true);
  });

  it("propage l'erreur backend quand le maximum de 3 onglets est atteint", async () => {
    vi.mocked(invoke).mockImplementation((command: string) => {
      if (command === "list_session_tabs") return Promise.resolve(rootTabs);
      if (command === "clone_agent_session") return Promise.reject(new Error("max tabs"));
      return Promise.resolve(rootTabs);
    });
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(rootTabs));

    await expect(result.current.cloneMessage({
      messageId: "m1",
      mode: "summary",
      operationId: "op-frontend",
    })).rejects.toThrow("max tabs");
  });

  it("ignore les onglets de clone retournés pour une autre racine", async () => {
    const wrongRootTabs: SessionTabs = {
      active_tab_id: "branch-1",
      tabs: [
        { tab_id: "main", session_id: "clone-root", label: "Main", is_main: true },
        { tab_id: "branch-1", session_id: "nested", label: "Branche 1", is_main: false },
      ],
    };
    vi.mocked(invoke).mockImplementation((command: string) => {
      if (command === "list_session_tabs") return Promise.resolve(rootTabs);
      if (command === "clone_agent_session") {
        return Promise.resolve({
          ...cloneResult,
          root_session_id: "clone-root",
          tabs: wrongRootTabs,
        });
      }
      return Promise.resolve(rootTabs);
    });
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(rootTabs));

    await act(async () => {
      await result.current.cloneMessage({
        messageId: "m1",
        mode: "cut",
        operationId: "op-frontend",
      });
    });

    expect(result.current.tabs).toEqual(rootTabs);
  });

  it("crée et lie une branche git de clone", async () => {
    const linkedTabs: SessionTabs = {
      ...cloneTabs,
      tabs: cloneTabs.tabs.map((tab) =>
        tab.session_id === "clone" ? { ...tab, git_branch: "clone-11111111" } : tab),
    };
    vi.mocked(invoke).mockImplementation((command: string) => {
      if (command === "list_session_tabs") return Promise.resolve(cloneTabs);
      if (command === "create_clone_git_branch") {
        return Promise.resolve({ branch_name: "clone-11111111", tabs: linkedTabs });
      }
      return Promise.resolve(rootTabs);
    });
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(cloneTabs));

    let branchName = "";
    await act(async () => {
      branchName = await result.current.createCloneGitBranch("/repo", "clone");
    });
    expect(branchName).toBe("clone-11111111");

    expect(invoke).toHaveBeenCalledWith("create_clone_git_branch", {
      sessionId: "root",
      cloneSessionId: "clone",
      path: "/repo",
    });
    await waitFor(() => expect(result.current.tabs).toEqual(linkedTabs));
  });

  it("nettoie la branche git avant de fermer un onglet clone", async () => {
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(rootTabs));

    await act(async () => {
      await result.current.closeTabWithGitCleanup("branch-1", "/repo", "main");
    });

    expect(invoke).toHaveBeenCalledWith("close_session_tab_and_cleanup_git_branch", {
      sessionId: "root",
      tabId: "branch-1",
      path: "/repo",
      fallbackBranch: "main",
    });
  });

  it("sauvegarde le checkpoint de branche principale dans les onglets", async () => {
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(rootTabs));

    await act(async () => {
      await result.current.saveMainCheckpointBranch("main");
    });

    expect(invoke).toHaveBeenCalledWith("save_session_tabs", {
      sessionId: "root",
      tabs: { ...rootTabs, main_checkpoint_branch: "main" },
    });
  });

  it.each([
    ["Ctrl", { code: "Digit2", key: "2", ctrlKey: true }],
    ["Cmd", { code: "Digit2", key: "2", metaKey: true }],
  ])("sélectionne le deuxième onglet avec %s + 2", async (_label, keyboard) => {
    const tabsWithMainActive = { ...cloneTabs, active_tab_id: "main" };
    vi.mocked(invoke).mockImplementation((command: string, args?: unknown) => {
      if (command === "list_session_tabs") return Promise.resolve(tabsWithMainActive);
      if (command === "save_session_tabs") {
        return Promise.resolve((args as { tabs: SessionTabs }).tabs);
      }
      return Promise.resolve(rootTabs);
    });
    const { result } = renderHook(() => useSessionTabs("root"));
    await waitFor(() => expect(result.current.tabs).toEqual(tabsWithMainActive));

    fireEvent.keyDown(window, keyboard);

    await waitFor(() => expect(result.current.tabs?.active_tab_id).toBe("branch-1"));
    expect(invoke).toHaveBeenCalledWith("save_session_tabs", {
      sessionId: "root",
      tabs: { ...tabsWithMainActive, active_tab_id: "branch-1" },
    });
  });
});
