import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useTerminal } from "@/hooks/use-terminal";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const ready = (groupKey: string) => ({
  validGroupKeys: [groupKey],
  projectLoadState: "ready" as const,
});

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

  it("conserve les opérations de panneau et de groupe", async () => {
    const { result } = renderHook(() => useTerminal("test", "/test", ready("test")));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    expect(result.current.panelHeight).toBe(120);
    expect(result.current.getGroupPtyIds("nonexistent")).toEqual([]);

    act(() => { result.current.addTab("/test"); });
    expect(result.current.isOpen).toBe(true);
    act(() => { result.current.removeGroup("test"); });
    expect(result.current.tabs).toHaveLength(0);
    act(() => { result.current.togglePanel(); });
    expect(result.current.isOpen).toBe(false);
  });

  it("place la persistance en erreur sans écrire après une réponse IPC mal formée", async () => {
    invokeMock.mockImplementation((command: string) => Promise.resolve(
      command === "load_terminal_tabs" ? undefined : undefined,
    ));
    const { result } = renderHook(() => useTerminal("test", "/test", ready("test")));

    await waitFor(() => expect(result.current.persistenceStatus).toBe("error"));

    expect(invokeMock).not.toHaveBeenCalledWith("save_terminal_tabs", expect.anything());
  });
});
