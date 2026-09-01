import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useAgentLocalPanelNav } from "../use-agent-local-panel-nav";
import { DEFAULT_AGENT_LOCAL_NAV } from "@/types/navigation";
import type { useFileTree } from "../use-file-tree";

function fileTree(open: boolean) {
  return {
    open,
    setOpen: vi.fn(),
  } as unknown as ReturnType<typeof useFileTree>;
}

describe("useAgentLocalPanelNav", () => {
  it("restaure l'arborescence depuis le workspace", () => {
    const tree = fileTree(false);

    renderHook(() => useAgentLocalPanelNav({
      navState: { ...DEFAULT_AGENT_LOCAL_NAV, fileTreeOpen: true },
      fileTree: tree,
    }));

    expect(tree.setOpen).toHaveBeenCalledWith(true);
  });

  it("n'applique rien quand l'état local correspond déjà au workspace", () => {
    const tree = fileTree(false);

    renderHook(() => useAgentLocalPanelNav({
      navState: DEFAULT_AGENT_LOCAL_NAV,
      fileTree: tree,
    }));

    expect(tree.setOpen).not.toHaveBeenCalled();
  });

  it("ne referme pas une arborescence ouverte localement avant la publication du workspace", () => {
    const setOpen = vi.fn();
    const tree = { ...fileTree(false), setOpen };

    const { rerender } = renderHook(
      ({ open }) => useAgentLocalPanelNav({
        navState: DEFAULT_AGENT_LOCAL_NAV,
        fileTree: { ...tree, open },
      }),
      { initialProps: { open: false } },
    );

    setOpen.mockClear();
    rerender({ open: true });

    expect(setOpen).not.toHaveBeenCalledWith(false);
  });

  it("resynchronise l'arborescence quand la session change avec les mêmes valeurs", () => {
    const setOpen = vi.fn();
    const { rerender } = renderHook(
      ({ sessionId, open }) => useAgentLocalPanelNav({
        navState: { ...DEFAULT_AGENT_LOCAL_NAV, sessionId },
        fileTree: { ...fileTree(open), setOpen },
      }),
      { initialProps: { sessionId: "session-a", open: false } },
    );

    rerender({ sessionId: "session-b", open: true });

    expect(setOpen).toHaveBeenCalledWith(false);
  });
});
