import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTerminal } from "../use-terminal";

describe("useTerminal", () => {
  const GROUP_KEY = "test-project";
  const DEFAULT_CWD = "/Users/test/project";

  it("starts with no tabs, closed panel", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    expect(result.current.tabs).toEqual([]);
    expect(result.current.isOpen).toBe(false);
    expect(result.current.activeTabId).toBeNull();
  });

  it("addTab creates a tab with folder name as label", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.addTab("/Users/test/my-app"); });
    expect(result.current.tabs).toHaveLength(1);
    expect(result.current.tabs[0].label).toBe("my-app");
    expect(result.current.tabs[0].cwd).toBe("/Users/test/my-app");
    expect(result.current.isOpen).toBe(true);
  });

  it("addTab without cwd uses defaultCwd", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.addTab(); });
    expect(result.current.tabs[0].cwd).toBe(DEFAULT_CWD);
    expect(result.current.tabs[0].label).toBe("project");
  });

  it("closeTab removes the tab", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.addTab("/a"); });
    act(() => { result.current.addTab("/b"); });
    const idToClose = result.current.tabs[0].id;
    act(() => { result.current.closeTab(idToClose); });
    expect(result.current.tabs).toHaveLength(1);
    expect(result.current.tabs[0].cwd).toBe("/b");
  });

  it("closing last tab sets isOpen to false", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.addTab(); });
    const id = result.current.tabs[0].id;
    act(() => { result.current.closeTab(id); });
    expect(result.current.tabs).toHaveLength(0);
    expect(result.current.isOpen).toBe(false);
  });

  it("renameTab updates the label", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.addTab(); });
    const id = result.current.tabs[0].id;
    act(() => { result.current.renameTab(id, "My Terminal"); });
    expect(result.current.tabs[0].label).toBe("My Terminal");
  });

  it("reorderTabs moves tab from index 0 to 2", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.addTab("/a"); });
    act(() => { result.current.addTab("/b"); });
    act(() => { result.current.addTab("/c"); });
    act(() => { result.current.reorderTabs(0, 2); });
    expect(result.current.tabs.map((t) => t.cwd)).toEqual(["/b", "/c", "/a"]);
  });

  it("togglePanel opens and closes", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.addTab(); });
    expect(result.current.isOpen).toBe(true);
    act(() => { result.current.togglePanel(); });
    expect(result.current.isOpen).toBe(false);
    act(() => { result.current.togglePanel(); });
    expect(result.current.isOpen).toBe(true);
  });

  it("togglePanel without tabs does nothing", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.togglePanel(); });
    expect(result.current.isOpen).toBe(false);
  });

  it("resizePanel clamps to maxHeight", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.setMaxHeight(400); });
    act(() => { result.current.resizePanel(9999); });
    expect(result.current.panelHeight).toBe(400);
  });

  it("resizePanel clamps to minHeight", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.setMaxHeight(400); });
    act(() => { result.current.resizePanel(10); });
    expect(result.current.panelHeight).toBe(80);
  });

  it("setPtyId updates the ptyId of a tab", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.addTab(); });
    const id = result.current.tabs[0].id;
    act(() => { result.current.setPtyId(id, 42); });
    expect(result.current.tabs[0].ptyId).toBe(42);
  });

  it("closing active tab switches to next available", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD));
    act(() => { result.current.addTab("/a"); });
    act(() => { result.current.addTab("/b"); });
    act(() => { result.current.addTab("/c"); });
    const firstId = result.current.tabs[0].id;
    act(() => { result.current.setActiveTab(firstId); });
    act(() => { result.current.closeTab(firstId); });
    expect(result.current.activeTabId).toBeTruthy();
    expect(result.current.tabs).toHaveLength(2);
  });
});
