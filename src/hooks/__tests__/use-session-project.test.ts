import { act, renderHook, waitFor } from "@testing-library/react";
import { createElement, type ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { AGENT_SESSIONS_CHANGED } from "../agent-session-events";
import { useSessionProject } from "../use-session-project";
import type { AgentSession, Project } from "@/types/agent";
import { AppNavigationActionsProvider } from "../use-app-navigation-actions";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

function wrapper({ children }: { children: ReactNode }) {
  return createElement(AppNavigationActionsProvider, {
    openFileAccessSettings: vi.fn(),
  }, children);
}

const project: Project = {
  id: "project-1",
  name: "Projet supprimé",
  path: "/project/gone",
  order: 0,
  created_at: "2026-07-12T00:00:00Z",
};

function session(projectId: string | undefined, workingDir: string): AgentSession {
  return {
    id: "session-1",
    name: "Test",
    model: "llama3",
    provider: "ollama",
    thinking_enabled: false,
    accumulated_tokens: 0,
    messages: [],
    created_at: "2026-07-12T00:00:00Z",
    project_id: projectId,
    working_dir: workingDir,
  };
}

describe("useSessionProject", () => {
  it("affiche immédiatement le dossier de secours quand le projet référencé manque", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(session("missing-project", "/project"));

    const { result } = renderHook(
      () => useSessionProject("session-1", [], vi.fn(), true),
      { wrapper },
    );

    await waitFor(() => expect(invoke).toHaveBeenCalled());
    await waitFor(() => expect(result.current.selectedProject?.path).toBe("/project"));
    expect(result.current.selectedProject?.name).toBe("project");
    expect(result.current.hidden).toBe(false);
  });

  it("affiche le dossier de session après un Switcher", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(session(project.id, project.path))
      .mockResolvedValueOnce(session(undefined, "/project"));
    const { result } = renderHook(
      () => useSessionProject("session-1", [project], vi.fn(), true),
      { wrapper },
    );
    await waitFor(() => expect(result.current.selectedProject?.id).toBe(project.id));

    void act(() => {
      window.dispatchEvent(new Event(AGENT_SESSIONS_CHANGED));
    });

    await waitFor(() => expect(result.current.selectedProject?.path).toBe("/project"));
    expect(result.current.selectedProject?.name).toBe("project");
    expect(result.current.hidden).toBe(false);
  });

  it("n'affiche jamais le workspace automatique sous le champ de saisie", async () => {
    const managed = session(undefined, "/private/app/session-workspaces/test/work");
    managed.working_dir_managed = true;
    vi.mocked(invoke).mockResolvedValueOnce(managed);

    const { result } = renderHook(
      () => useSessionProject("session-1", [], vi.fn(), true),
      { wrapper },
    );

    await waitFor(() => expect(invoke).toHaveBeenCalled());
    await act(async () => {});
    expect(result.current.selectedProject).toBeUndefined();
    expect(result.current.workingDir).toBe("");
    expect(result.current.hidden).toBe(true);
  });
});
