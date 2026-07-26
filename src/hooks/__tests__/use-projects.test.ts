import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { AGENT_SESSIONS_CHANGED } from "../agent-session-events";
import { useProjects } from "../use-projects";
import type { Project } from "@/types/agent";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const recovered: Project = {
  id: "recovered",
  name: "Projects",
  path: "/Users/test/Projects",
  order: 0,
  created_at: "2026-07-12T00:00:00Z",
};

describe("useProjects", () => {
  it("recharge les projets après la récupération d'une session", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([recovered]);
    const { result } = renderHook(() => useProjects());
    await waitFor(() => expect(result.current.projects).toEqual([]));

    void act(() => window.dispatchEvent(new Event(AGENT_SESSIONS_CHANGED)));

    await waitFor(() => expect(result.current.projects).toEqual([recovered]));
    expect(invoke).toHaveBeenCalledTimes(2);
  });
});
