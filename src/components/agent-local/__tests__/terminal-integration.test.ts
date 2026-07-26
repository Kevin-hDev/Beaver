import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTerminal } from "@/hooks/use-terminal";

describe("terminal integration - isolation par projet", () => {
  it("tabs are isolated per groupKey", () => {
    const { result } = renderHook(() => useTerminal("project-a", "/a"));

    act(() => { result.current.addTab("/a/dir"); });
    expect(result.current.tabs).toHaveLength(1);

    const allTabs = result.current.allTabs();
    expect(allTabs).toHaveLength(1);
    expect(allTabs[0].groupKey).toBe("project-a");
  });

  it("multiple addTab calls create distinct tabs", () => {
    const { result } = renderHook(() => useTerminal("test", "/test"));
    act(() => { result.current.addTab("/a"); });
    act(() => { result.current.addTab("/b"); });
    act(() => { result.current.addTab("/c"); });

    const ids = result.current.tabs.map((t) => t.id);
    const unique = new Set(ids);
    expect(unique.size).toBe(3);
  });

  it("reorder does not lose tabs", () => {
    const { result } = renderHook(() => useTerminal("test", "/test"));
    act(() => { result.current.addTab("/a"); });
    act(() => { result.current.addTab("/b"); });
    act(() => { result.current.addTab("/c"); });

    const before = result.current.tabs.map((t) => t.id).sort();
    act(() => { result.current.reorderTabs(0, 2); });
    const after = result.current.tabs.map((t) => t.id).sort();
    expect(after).toEqual(before);
  });

  it("default panel height is 120px", () => {
    const { result } = renderHook(() => useTerminal("test", "/test"));
    expect(result.current.panelHeight).toBe(120);
  });

  it("removeGroup deletes a group", () => {
    const { result } = renderHook(() => useTerminal("to-delete", "/tmp"));
    act(() => { result.current.addTab("/tmp"); });
    expect(result.current.tabs).toHaveLength(1);

    act(() => { result.current.removeGroup("to-delete"); });
    expect(result.current.tabs).toHaveLength(0);
  });

  it("getGroupPtyIds returns empty for unknown group", () => {
    const { result } = renderHook(() => useTerminal("test", "/test"));
    expect(result.current.getGroupPtyIds("nonexistent")).toEqual([]);
  });

  it("closing last tab closes panel", () => {
    const { result } = renderHook(() => useTerminal("test", "/test"));
    act(() => { result.current.addTab("/test"); });
    expect(result.current.isOpen).toBe(true);

    const tabId = result.current.tabs[0].id;
    act(() => { result.current.closeTab(tabId); });
    expect(result.current.isOpen).toBe(false);
  });

  it("toggle on empty group does not open panel", () => {
    const { result } = renderHook(() => useTerminal("empty", "/test"));
    act(() => { result.current.togglePanel(); });
    expect(result.current.isOpen).toBe(false);
  });
});
