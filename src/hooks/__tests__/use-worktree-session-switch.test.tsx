import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  findProjectByPath,
  normalizeProjectPath,
  useWorktreeSessionSwitch,
} from "../use-worktree-session-switch";
import type { Project } from "@/types/agent";

function project(id: string, path: string): Project {
  return {
    id,
    path,
    name: id,
    order: 0,
    created_at: "2026-05-17T00:00:00Z",
  };
}

describe("useWorktreeSessionSwitch", () => {
  it("reuses an existing project and preserves the current model", async () => {
    const existing = project("project-1", "/tmp/worktree");
    const addProject = vi.fn<(path: string) => Promise<Project>>();
    const newSession = vi.fn();
    const { result } = renderHook(() =>
      useWorktreeSessionSwitch({
        projects: [existing],
        model: "gpt-5.5",
        provider: "openai",
        onAddProject: addProject,
        onNewSessionInProject: newSession,
      }),
    );

    act(() => result.current.request("/tmp/worktree/", "feature"));
    await act(async () => { await result.current.createSession(); });

    expect(addProject).not.toHaveBeenCalled();
    expect(newSession).toHaveBeenCalledWith("gpt-5.5", "openai", "project-1");
    expect(result.current.pending).toBeNull();
  });

  it("adds a missing project before creating the session", async () => {
    const added = project("project-2", "/tmp/new-worktree");
    const addProject = vi.fn<(path: string) => Promise<Project>>().mockResolvedValue(added);
    const newSession = vi.fn();
    const { result } = renderHook(() =>
      useWorktreeSessionSwitch({
        projects: [],
        model: "gemma4:latest",
        provider: "ollama",
        onAddProject: addProject,
        onNewSessionInProject: newSession,
      }),
    );

    act(() => result.current.request("/tmp/new-worktree", "worktree-agent"));
    await act(async () => { await result.current.createSession(); });

    expect(addProject).toHaveBeenCalledWith("/tmp/new-worktree");
    expect(newSession).toHaveBeenCalledWith("gemma4:latest", "ollama", "project-2");
  });

  it("normalizes simple trailing separators when matching projects", () => {
    const existing = project("project-1", "/tmp/worktree");

    expect(normalizeProjectPath("/tmp/worktree/")).toBe("/tmp/worktree");
    expect(findProjectByPath([existing], "/tmp/worktree/")?.id).toBe("project-1");
  });
});
