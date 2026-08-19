import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { AGENT_SESSIONS_CHANGED } from "../agent-session-events";
import { useProjects } from "../use-projects";
import type { Project } from "@/types/agent";

vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));

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

  it("conserve les projets visibles et traduit une panne du stockage", async () => {
    const { showToast } = await import("@/lib/toast-emitter");
    vi.mocked(invoke)
      .mockResolvedValueOnce([recovered])
      .mockRejectedValueOnce("project-store-unavailable");
    const { result } = renderHook(() => useProjects());
    await waitFor(() => expect(result.current.projects).toEqual([recovered]));

    await act(async () => result.current.refresh());

    expect(result.current.projects).toEqual([recovered]);
    expect(showToast).toHaveBeenCalledWith(
      "Beaver cannot read or save projects. Check the disk and permissions, then try again.",
      "error",
    );
  });
});
