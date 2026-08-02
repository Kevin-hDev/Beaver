import { act, renderHook } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import { AppNavigationActionsProvider } from "../use-app-navigation-actions";
import { useSessionActions, type SessionActionsDeps } from "../use-session-actions";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const project = {
  id: "project-1",
  name: "Projet",
  path: "/project/forbidden",
  order: 0,
  created_at: "2026-08-02T00:00:00Z",
};

function dependencies(create = vi.fn()): SessionActionsDeps {
  return {
    create,
    rename: vi.fn(),
    defaultModel: "model",
    defaultProvider: "provider",
    welcomeModel: null,
    setWelcomeModel: vi.fn(),
    projectsHook: { projects: [project] },
    onSessionChange: vi.fn(),
  };
}

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <AppNavigationActionsProvider openFileAccessSettings={vi.fn()}>
    {children}
  </AppNavigationActionsProvider>
);

describe("useSessionActions", () => {
  it("bloque visiblement la création dans un projet interdit", async () => {
    vi.mocked(invoke).mockResolvedValue({
      allowed: false,
      allowed_paths: ["/project/allowed"],
    });
    const deps = dependencies();
    const { result } = renderHook(() => useSessionActions(deps), { wrapper });

    await act(() => result.current.handleCreateInProject(project.id));

    expect(deps.create).not.toHaveBeenCalled();
    expect(result.current.directoryAccessPrompt?.allowedPaths).toEqual(["/project/allowed"]);
  });

  it("crée la session après validation du backend", async () => {
    vi.mocked(invoke).mockResolvedValue({ allowed: true, allowed_paths: ["/"] });
    const create = vi.fn().mockResolvedValue({ id: "session-1" });
    const deps = dependencies(create);
    const { result } = renderHook(() => useSessionActions(deps), { wrapper });

    await act(() => result.current.handleCreateInProjectWithModel("m", "p", project.id));

    expect(create).toHaveBeenCalledWith("agentLocal.newSession", "m", "p", project.id);
    expect(deps.onSessionChange).toHaveBeenCalledWith("session-1");
  });
});
