import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { useConversationCollapseState } from "../use-conversation-collapse-state";

const STORAGE_KEY = "clgo-conversation-collapse-v1";

describe("useConversationCollapseState", () => {
  beforeEach(() => localStorage.clear());

  it("ouvre toutes les sections par défaut", () => {
    const { result } = renderHook(() => useConversationCollapseState());

    expect(result.current.projectsCollapsed).toBe(false);
    expect(result.current.discussionsCollapsed).toBe(false);
    expect(result.current.collapsedProjects.size).toBe(0);
  });

  it("restaure les sections et projets repliés", () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      projectsCollapsed: true,
      discussionsCollapsed: true,
      collapsedProjectIds: ["project-1", "project-2"],
    }));

    const { result } = renderHook(() => useConversationCollapseState());

    expect(result.current.projectsCollapsed).toBe(true);
    expect(result.current.discussionsCollapsed).toBe(true);
    expect([...result.current.collapsedProjects]).toEqual(["project-1", "project-2"]);
  });

  it("sauvegarde les changements pour le prochain montage", () => {
    const first = renderHook(() => useConversationCollapseState());
    act(() => {
      first.result.current.toggleProjects();
      first.result.current.toggleDiscussions();
      first.result.current.toggleProject("project-1");
    });
    first.unmount();

    const second = renderHook(() => useConversationCollapseState());

    expect(second.result.current.projectsCollapsed).toBe(true);
    expect(second.result.current.discussionsCollapsed).toBe(true);
    expect(second.result.current.collapsedProjects.has("project-1")).toBe(true);
  });

  it("ignore les données invalides et borne la liste restaurée", () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      projectsCollapsed: false,
      discussionsCollapsed: false,
      collapsedProjectIds: [
        "../invalid",
        ...Array.from({ length: 300 }, (_, index) => `project-${index}`),
      ],
    }));

    const { result } = renderHook(() => useConversationCollapseState());

    expect(result.current.collapsedProjects.has("../invalid")).toBe(false);
    expect(result.current.collapsedProjects.size).toBe(255);
  });

  it("revient à l'état par défaut si le stockage est illisible", () => {
    localStorage.setItem(STORAGE_KEY, "{invalide");

    const { result } = renderHook(() => useConversationCollapseState());

    expect(result.current.projectsCollapsed).toBe(false);
    expect(result.current.discussionsCollapsed).toBe(false);
    expect(result.current.collapsedProjects.size).toBe(0);
  });
});
