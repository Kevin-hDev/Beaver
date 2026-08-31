import { act, renderHook, waitFor } from "@testing-library/react";
import { useCallback, useState } from "react";
import { describe, expect, it, vi, type Mock } from "vitest";
import { DEFAULT_APP_NAV } from "@/types/navigation";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";
import type { TerminalGroup, TerminalTab } from "../terminal-types";
import { useAgentLocalControlledTerminal } from "../use-agent-local-controlled-terminal";

type ControlledWithGroupClose = ReturnType<typeof useAgentLocalControlledTerminal> & {
  closeTabInGroup: (groupKey: string, tabId: string) => void;
};

function tab(id: string): TerminalTab {
  return { id, ptyId: null, ptyToken: null, label: id, hasActivity: false };
}

function terminalFixture(overrides: Record<string, unknown> = {}) {
  return {
    tabs: [] as TerminalTab[],
    activeTabId: null as string | null,
    isOpen: false,
    panelHeight: 120,
    persistenceStatus: "healthy" as const,
    allTabs: vi.fn(() => []),
    addTab: vi.fn((): string | null => "new-tab"),
    closeTab: vi.fn(),
    closeTabInGroup: vi.fn(),
    setActiveTab: vi.fn(),
    renameTab: vi.fn(() => true),
    reorderTabs: vi.fn(),
    togglePanel: vi.fn(),
    setPtyId: vi.fn(),
    setTabActivity: vi.fn(),
    resizePanel: vi.fn(),
    setMaxHeight: vi.fn(),
    removeGroup: vi.fn(),
    getGroupPtyIds: vi.fn(() => []),
    getGroupPtyEntries: vi.fn(() => []),
    groupKey: "project-a",
    ...overrides,
  };
}

function closeGroup(
  groups: Map<string, TerminalGroup>,
  groupKey: string,
  tabId: string,
): Map<string, TerminalGroup> {
  const group = groups.get(groupKey);
  if (!group) return groups;
  const closedIndex = group.tabs.findIndex(({ id }) => id === tabId);
  if (closedIndex < 0) return groups;
  const tabs = group.tabs.filter(({ id }) => id !== tabId);
  const activeTabId = group.activeTabId === tabId
    ? tabs[Math.min(closedIndex, tabs.length - 1)]?.id ?? null
    : group.activeTabId;
  const next = new Map(groups);
  next.set(groupKey, { tabs, activeTabId });
  return next;
}

function renderStatefulControlled({
  initialGroups,
  groupKey = "project-a",
  onNavChange = vi.fn(),
}: {
  initialGroups: Map<string, TerminalGroup>;
  groupKey?: string;
  onNavChange?: Mock<(partial: DeepPartial<AgentLocalNavState>) => void>;
}) {
  const rendered = renderHook(
    ({ activeGroupKey }) => {
      const [groups, setGroups] = useState(initialGroups);
      const group = groups.get(activeGroupKey) ?? { tabs: [], activeTabId: null };
      const closeTabInGroup = useCallback((targetGroupKey: string, tabId: string) => {
        setGroups((previous) => closeGroup(previous, targetGroupKey, tabId));
      }, []);
      const closeTab = useCallback((tabId: string) => {
        closeTabInGroup(activeGroupKey, tabId);
      }, [activeGroupKey, closeTabInGroup]);
      const terminalState = terminalFixture({
        tabs: group.tabs,
        activeTabId: group.activeTabId,
        groupKey: activeGroupKey,
        closeTab,
        closeTabInGroup,
      });
      return useAgentLocalControlledTerminal({
        navState: { ...DEFAULT_APP_NAV.agentLocal, terminalOpen: true },
        terminalState,
        terminalCwd: "/project",
        onNavChange,
      });
    },
    { initialProps: { activeGroupKey: groupKey } },
  );
  return { ...rendered, onNavChange };
}

describe("useAgentLocalControlledTerminal", () => {
  it("ne remplace jamais l'onglet actif possédé par useTerminal", () => {
    const terminalState = terminalFixture({ activeTabId: "runtime-tab" });
    const navState = { ...DEFAULT_APP_NAV.agentLocal, terminalOpen: true };

    const { result } = renderHook(() => useAgentLocalControlledTerminal({
      navState,
      terminalState,
      terminalCwd: "/project",
      onNavChange: vi.fn(),
    }));

    expect(result.current.activeTabId).toBe("runtime-tab");
  });

  it("fermer le dernier onglet ferme aussi la navigation après le rendu", async () => {
    const { result, onNavChange } = renderStatefulControlled({
      initialGroups: new Map([
        ["project-a", { tabs: [tab("one")], activeTabId: "one" }],
      ]),
    });

    act(() => result.current.closeTab("one"));

    await waitFor(() => {
      expect(onNavChange).toHaveBeenCalledWith({ terminalOpen: false });
    });
  });

  it("fermer l'onglet actif parmi deux garde le panneau et sélectionne le suivant", () => {
    const { result, onNavChange } = renderStatefulControlled({
      initialGroups: new Map([
        ["project-a", { tabs: [tab("one"), tab("two")], activeTabId: "one" }],
      ]),
    });

    act(() => result.current.closeTab("one"));

    expect(result.current.tabs.map(({ id }) => id)).toEqual(["two"]);
    expect(result.current.activeTabId).toBe("two");
    expect(onNavChange).not.toHaveBeenCalledWith({ terminalOpen: false });
  });

  it("passer d'un groupe non vide à un autre vide ne ferme pas la navigation", () => {
    const { rerender, onNavChange } = renderStatefulControlled({
      initialGroups: new Map([
        ["project-a", { tabs: [tab("one")], activeTabId: "one" }],
        ["project-b", { tabs: [], activeTabId: null }],
      ]),
    });

    rerender({ activeGroupKey: "project-b" });

    expect(onNavChange).not.toHaveBeenCalledWith({ terminalOpen: false });
  });

  it("vider un groupe de fond ne ferme pas le panneau courant", () => {
    const { result, onNavChange } = renderStatefulControlled({
      initialGroups: new Map([
        ["project-a", { tabs: [tab("one")], activeTabId: "one" }],
        ["project-b", { tabs: [tab("two")], activeTabId: "two" }],
      ]),
    });

    act(() => {
      (result.current as ControlledWithGroupClose).closeTabInGroup("project-b", "two");
    });

    expect(result.current.tabs.map(({ id }) => id)).toEqual(["one"]);
    expect(onNavChange).not.toHaveBeenCalledWith({ terminalOpen: false });
  });

  it("ouvre un groupe vide seulement si la création d'onglet réussit", () => {
    const onNavChange = vi.fn();
    const addTab = vi.fn(() => "new-tab");
    const terminalState = terminalFixture({ addTab });
    const { result } = renderHook(() => useAgentLocalControlledTerminal({
      navState: DEFAULT_APP_NAV.agentLocal,
      terminalState,
      terminalCwd: "/project",
      onNavChange,
    }));

    act(() => result.current.togglePanel());

    expect(addTab).toHaveBeenCalledWith("/project");
    expect(onNavChange).toHaveBeenCalledWith({ terminalOpen: true });
  });

  it("reste fermé si la création du premier onglet est refusée", () => {
    const onNavChange = vi.fn();
    const terminalState = terminalFixture({ addTab: vi.fn(() => null) });
    const { result } = renderHook(() => useAgentLocalControlledTerminal({
      navState: DEFAULT_APP_NAV.agentLocal,
      terminalState,
      terminalCwd: "/project",
      onNavChange,
    }));

    act(() => result.current.togglePanel());

    expect(onNavChange).not.toHaveBeenCalledWith({ terminalOpen: true });
  });
});
