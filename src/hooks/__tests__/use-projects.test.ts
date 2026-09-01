import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { AGENT_SESSIONS_CHANGED } from "../agent-session-events";
import { useProjects } from "../use-projects";
import type { Project } from "@/types/agent";

vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const invokeMock = vi.mocked(invoke);

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((ok, fail) => {
    resolve = ok;
    reject = fail;
  });
  return { promise, resolve, reject };
}

const recovered: Project = {
  id: "recovered",
  name: "Projects",
  path: "/Users/test/Projects",
  order: 0,
  created_at: "2026-07-12T00:00:00Z",
};

describe("useProjects", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("reste loading jusqu'à la réponse du registre", async () => {
    const pending = deferred<Project[]>();
    invokeMock.mockReturnValueOnce(pending.promise);
    const { result } = renderHook(() => useProjects());

    expect(result.current.loadState).toBe("loading");
    pending.resolve([]);
    await waitFor(() => expect(result.current.loadState).toBe("ready"));
  });

  it("une erreur ne transforme pas le catalogue en liste vide prête", async () => {
    const { showToast } = await import("@/lib/toast-emitter");
    invokeMock.mockRejectedValueOnce(new Error("private path"));
    const { result } = renderHook(() => useProjects());

    await waitFor(() => expect(result.current.loadState).toBe("error"));
    expect(result.current.projects).toEqual([]);
    expect(showToast).toHaveBeenCalledWith(
      "Operation failed. Please try again.",
      "error",
    );
    expect(showToast).not.toHaveBeenCalledWith(
      expect.stringContaining("private path"),
      expect.anything(),
    );
  });

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
