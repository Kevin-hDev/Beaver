import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useProjects } from "@/hooks/use-projects";
import { useTerminal } from "@/hooks/use-terminal";
import type { Project } from "@/types/agent";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const ready = (groupKey: string) => ({
  validGroupKeys: [groupKey],
  projectLoadState: "ready" as const,
});

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((ok) => { resolve = ok; });
  return { promise, resolve };
}

function useTerminalWithProjects() {
  const projects = useProjects();
  const terminal = useTerminal("project-a", "/a", {
    validGroupKeys: projects.projects.map((project) => project.id),
    projectLoadState: projects.loadState,
  });
  return { terminal, projectLoadState: projects.loadState };
}

describe("terminal integration - isolation par projet", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => Promise.resolve(
      command === "load_terminal_tabs" ? { version: 1, groups: {} } : undefined,
    ));
  });

  it("isole les onglets par groupKey", async () => {
    const { result } = renderHook(() => useTerminal("project-a", "/a", ready("project-a")));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));

    act(() => { result.current.addTab("/a/dir"); });

    expect(result.current.tabs).toHaveLength(1);
    expect(result.current.allTabs()).toHaveLength(1);
    expect(result.current.allTabs()[0].groupKey).toBe("project-a");
  });

  it("crée des identifiants distincts sans perdre d'onglet au réordonnancement", async () => {
    const { result } = renderHook(() => useTerminal("test", "/test", ready("test")));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    for (const cwd of ["/a", "/b", "/c"]) {
      act(() => { result.current.addTab(cwd); });
    }
    const before = result.current.tabs.map((tab) => tab.id).sort();

    act(() => { result.current.reorderTabs(0, 2); });

    expect(new Set(before).size).toBe(3);
    expect(result.current.tabs.map((tab) => tab.id).sort()).toEqual(before);
  });

  it("conserve les opérations de hauteur et de groupe", async () => {
    const { result } = renderHook(() => useTerminal("test", "/test", ready("test")));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    expect(result.current.panelHeight).toBe(120);
    expect(result.current.getGroupPtyIds("nonexistent")).toEqual([]);

    act(() => { result.current.addTab("/test"); });
    act(() => { result.current.removeGroup("test"); });
    expect(result.current.tabs).toHaveLength(0);
  });

  it("place la persistance en erreur sans écrire après une réponse IPC mal formée", async () => {
    invokeMock.mockImplementation((command: string) => Promise.resolve(
      command === "load_terminal_tabs" ? undefined : undefined,
    ));
    const { result } = renderHook(() => useTerminal("test", "/test", ready("test")));

    await waitFor(() => expect(result.current.persistenceStatus).toBe("error"));

    expect(invokeMock).not.toHaveBeenCalledWith("save_terminal_tabs", expect.anything());
  });

  it("conserve project-a pendant deux cycles avant la réponse de list_projects", async () => {
    const pendingProjects = deferred<Project[]>();
    invokeMock.mockImplementation((command: string) => {
      if (command === "load_terminal_tabs") {
        return Promise.resolve({ version: 1, groups: { "project-a": [{ label: "build" }] } });
      }
      if (command === "list_projects") return pendingProjects.promise;
      return Promise.resolve(undefined);
    });
    const { result, rerender } = renderHook(() => useTerminalWithProjects());
    await waitFor(() => expect(result.current.terminal.persistenceStatus).toBe("healthy"));

    rerender();
    expect(result.current.terminal.tabs).toHaveLength(1);
    rerender();
    expect(result.current.terminal.tabs).toHaveLength(1);
    expect(result.current.projectLoadState).toBe("loading");
    expect(invokeMock).not.toHaveBeenCalledWith("save_terminal_tabs", expect.anything());

    await act(async () => {
      pendingProjects.resolve([{
        id: "project-a",
        name: "Project A",
        path: "/a",
        order: 0,
        created_at: "2026-08-31T00:00:00Z",
      }]);
      await pendingProjects.promise;
    });

    await waitFor(() => expect(result.current.projectLoadState).toBe("ready"));
    expect(result.current.terminal.tabs).toHaveLength(1);
    expect(result.current.terminal.tabs[0].label).toBe("build");
    expect(invokeMock).not.toHaveBeenCalledWith("save_terminal_tabs", expect.anything());
  });
});
