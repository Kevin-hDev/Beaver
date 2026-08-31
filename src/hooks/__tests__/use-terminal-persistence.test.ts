import { act, renderHook, waitFor } from "@testing-library/react";
import { StrictMode, useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { showToast } from "@/lib/toast-emitter";
import { useTerminalPersistence } from "../use-terminal-persistence";
import type { TerminalGroup } from "../terminal-types";

const { loadMock, saveMock } = vi.hoisted(() => ({
  loadMock: vi.fn(),
  saveMock: vi.fn(),
}));

vi.mock("../terminal-persistence", () => ({
  loadSavedGroups: loadMock,
  saveGroups: saveMock,
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

function useHarness(canSave: boolean) {
  const [groups, setGroups] = useState<Map<string, TerminalGroup>>(new Map());
  const persistence = useTerminalPersistence({ groups, setGroups, canSave });
  return { groups, setGroups, ...persistence };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((ok) => { resolve = ok; });
  return { promise, resolve };
}

describe("useTerminalPersistence", () => {
  beforeEach(() => {
    loadMock.mockReset();
    saveMock.mockReset();
    vi.mocked(showToast).mockReset();
  });

  it("ne lance qu'un chargement sous StrictMode et garde son résultat comme seule autorité", async () => {
    const first = deferred<{ version: 1; groups: { project: { label: string }[] } }>();
    loadMock
      .mockImplementationOnce(() => first.promise)
      .mockResolvedValue({ version: 1, groups: { project: [{ label: "second" }] } });
    const { result } = renderHook(() => useHarness(false), { wrapper: StrictMode });

    expect(loadMock).toHaveBeenCalledOnce();
    first.resolve({ version: 1, groups: { project: [{ label: "first" }] } });
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    expect(result.current.groups.get("project")?.tabs[0].label).toBe("first");
  });

  it("devient sain seulement après restauration du document durable sans chemin", async () => {
    loadMock.mockResolvedValue({
      version: 1,
      groups: { project: [{ label: "build" }] },
    });
    const { result } = renderHook(() => useHarness(false));

    expect(result.current.persistenceStatus).toBe("loading");
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    expect(result.current.loaded).toBe(true);
    expect(result.current.groups.get("project")?.tabs[0]).toMatchObject({
      ptyId: null,
      ptyToken: null,
      label: "build",
      hasActivity: false,
    });
    expect(result.current.groups.get("project")?.tabs[0]).not.toHaveProperty("cwd");
    expect(saveMock).not.toHaveBeenCalled();
  });

  it("bloque les sauvegardes et demande un redémarrage après une lecture invalide", async () => {
    loadMock.mockRejectedValue(new Error("terminal-tabs-invalid"));
    const { result } = renderHook(() => useHarness(true));

    await waitFor(() => expect(result.current.persistenceStatus).toBe("error"));
    expect(result.current.loaded).toBe(false);
    expect(showToast).toHaveBeenCalledOnce();
    expect(showToast).toHaveBeenCalledWith("terminal.tabsLoadFailed", "error");
    expect(saveMock).not.toHaveBeenCalled();
  });

  it("arrête définitivement la file et affiche une seule erreur après une écriture échouée", async () => {
    loadMock.mockResolvedValue({ version: 1, groups: {} });
    saveMock.mockRejectedValue(new Error("disk detail"));
    const { result } = renderHook(() => useHarness(true));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));

    act(() => {
      result.current.setGroups(new Map([["project", {
        tabs: [{ id: "one", ptyId: null, ptyToken: null, label: "build", hasActivity: false }],
        activeTabId: "one",
      }]]));
    });

    await waitFor(() => expect(result.current.persistenceStatus).toBe("error"));
    act(() => {
      result.current.setGroups(new Map([["project", {
        tabs: [{ id: "two", ptyId: null, ptyToken: null, label: "test", hasActivity: false }],
        activeTabId: "two",
      }]]));
    });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(saveMock).toHaveBeenCalledOnce();
    expect(showToast).toHaveBeenCalledOnce();
    expect(showToast).toHaveBeenCalledWith("terminal.tabsSaveFailed", "error");
  });
});
