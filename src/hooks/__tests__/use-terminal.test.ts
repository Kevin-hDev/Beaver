import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { showToast } from "@/lib/toast-emitter";
import { useTerminal } from "../use-terminal";

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

const GROUP_KEY = "test-project";
const DEFAULT_CWD = "/Users/test/project";
const ready = (validGroupKeys = [GROUP_KEY]) => ({
  validGroupKeys,
  projectLoadState: "ready" as const,
});

describe("useTerminal", () => {
  beforeEach(() => {
    loadMock.mockReset();
    loadMock.mockResolvedValue({ version: 1, groups: {} });
    saveMock.mockReset();
    saveMock.mockResolvedValue(undefined);
    vi.mocked(showToast).mockReset();
  });

  it("démarre sans onglet et sans autorité d'ouverture du panneau", () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));

    expect(result.current.tabs).toEqual([]);
    expect(result.current.activeTabId).toBeNull();
    expect(result.current).not.toHaveProperty("isOpen");
    expect(result.current).not.toHaveProperty("togglePanel");
  });

  it("utilise le dossier par défaut comme libellé sans conserver son chemin", async () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));

    act(() => { result.current.addTab(); });

    expect(result.current.tabs[0].label).toBe("project");
    expect(result.current.tabs[0]).not.toHaveProperty("cwd");
  });

  it("ferme le dernier onglet", async () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    act(() => { result.current.addTab(); });
    const id = result.current.tabs[0].id;

    act(() => { result.current.closeTab(id); });

    expect(result.current.tabs).toHaveLength(0);
  });

  it("pince la hauteur du panneau entre le minimum et le maximum", async () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    act(() => { result.current.setMaxHeight(400); });

    act(() => { result.current.resizePanel(9999); });
    expect(result.current.panelHeight).toBe(400);
    act(() => { result.current.resizePanel(10); });
    expect(result.current.panelHeight).toBe(80);
  });

  it("crée un onglet sans conserver son chemin", async () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));

    let id: string | null = null;
    act(() => { id = result.current.addTab("/Users/test/my-app"); });

    expect(id).not.toBeNull();
    expect(result.current.tabs[0]).toMatchObject({ label: "my-app", hasActivity: false });
    expect(result.current.tabs[0]).not.toHaveProperty("cwd");
  });

  it("projette seulement les libellés vers le document durable", async () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));

    act(() => { result.current.addTab("/secret/worktree"); });

    await waitFor(() => expect(saveMock).toHaveBeenCalledOnce());
    expect(saveMock).toHaveBeenCalledWith({
      version: 1,
      groups: { [GROUP_KEY]: [{ label: "worktree" }] },
    });
  });

  it("conserve les groupes tant que les projets sont en chargement sans sauvegarder", async () => {
    loadMock.mockResolvedValue({
      version: 1,
      groups: { "project-a": [{ label: "build" }] },
    });
    const { result } = renderHook(() => useTerminal("project-a", "/a", {
      validGroupKeys: [],
      projectLoadState: "loading",
    }));

    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    expect(result.current.tabs).toHaveLength(1);
    expect(saveMock).not.toHaveBeenCalled();
  });

  it("conserve le projet restauré quand le chargement prêt le confirme", async () => {
    loadMock.mockResolvedValue({
      version: 1,
      groups: { "project-a": [{ label: "build" }] },
    });
    const { result, rerender } = renderHook(
      ({ projectLoadState }) => useTerminal("project-a", "/a", {
        validGroupKeys: ["project-a"],
        projectLoadState,
      }),
      { initialProps: { projectLoadState: "loading" as "loading" | "ready" } },
    );
    await waitFor(() => expect(result.current.tabs).toHaveLength(1));

    rerender({ projectLoadState: "ready" });

    expect(result.current.tabs).toHaveLength(1);
    expect(saveMock).not.toHaveBeenCalled();
  });

  it("retire un projet absent seulement à ready puis sauvegarde exactement une fois", async () => {
    loadMock.mockResolvedValue({
      version: 1,
      groups: { "project-a": [{ label: "build" }] },
    });
    const { result, rerender } = renderHook(
      ({ projectLoadState }) => useTerminal("project-a", "/a", {
        validGroupKeys: [],
        projectLoadState,
      }),
      { initialProps: { projectLoadState: "loading" as "loading" | "ready" } },
    );
    await waitFor(() => expect(result.current.tabs).toHaveLength(1));

    rerender({ projectLoadState: "ready" });

    await waitFor(() => expect(result.current.tabs).toHaveLength(0));
    await waitFor(() => expect(saveMock).toHaveBeenCalledOnce());
    expect(saveMock).toHaveBeenCalledWith({ version: 1, groups: {} });
  });

  it("normalise un renommage valide et refuse les libellés invalides", async () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    act(() => { result.current.addTab(); });
    const id = result.current.tabs[0].id;

    let renamed = false;
    act(() => { renamed = result.current.renameTab(id, "  Build  "); });
    expect(renamed).toBe(true);
    expect(result.current.tabs[0].label).toBe("Build");

    for (const invalid of [" ", "bad\nlabel", "é".repeat(257)]) {
      act(() => { renamed = result.current.renameTab(id, invalid); });
      expect(renamed).toBe(false);
      expect(result.current.tabs[0].label).toBe("Build");
    }
  });

  it("refuse le dix-septième onglet d'un groupe sans mutation", async () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    for (let index = 0; index < 16; index += 1) {
      act(() => { result.current.addTab(`/tab-${index}`); });
    }

    let refused: string | null = "unexpected";
    act(() => { refused = result.current.addTab("/tab-17"); });

    expect(refused).toBeNull();
    expect(result.current.tabs).toHaveLength(16);
    expect(showToast).toHaveBeenCalledOnce();
    expect(showToast).toHaveBeenCalledWith("terminal.tabLimitReached", "error");
  });

  it("refuse le 257e onglet total sans créer un nouveau groupe", async () => {
    const keys = Array.from({ length: 17 }, (_, index) => `group-${index}`);
    const { result, rerender } = renderHook(
      ({ groupKey }) => useTerminal(groupKey, "/root", ready(keys)),
      { initialProps: { groupKey: keys[0] } },
    );
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    for (const groupKey of keys.slice(0, 16)) {
      rerender({ groupKey });
      for (let tab = 0; tab < 16; tab += 1) {
        act(() => { result.current.addTab(`/${groupKey}-${tab}`); });
      }
    }
    rerender({ groupKey: keys[16] });

    let refused: string | null = "unexpected";
    act(() => { refused = result.current.addTab("/overflow"); });

    expect(refused).toBeNull();
    expect(result.current.tabs).toHaveLength(0);
    expect(showToast).toHaveBeenCalledOnce();
  });

  it("refuse le 129e groupe durable sans mutation", async () => {
    const keys = Array.from({ length: 129 }, (_, index) => `group-${index}`);
    const { result, rerender } = renderHook(
      ({ groupKey }) => useTerminal(groupKey, "/root", ready(keys)),
      { initialProps: { groupKey: keys[0] } },
    );
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    for (const groupKey of keys.slice(0, 128)) {
      rerender({ groupKey });
      act(() => { result.current.addTab(`/${groupKey}`); });
    }
    rerender({ groupKey: keys[128] });

    let refused: string | null = "unexpected";
    act(() => { refused = result.current.addTab("/overflow"); });

    expect(refused).toBeNull();
    expect(result.current.tabs).toHaveLength(0);
    expect(showToast).toHaveBeenCalledOnce();
  });

  it("ignore les réordonnancements invalides sans changer la liste", async () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    for (const cwd of ["/a", "/b", "/c"]) {
      act(() => { result.current.addTab(cwd); });
    }
    const before = result.current.tabs;

    for (const [from, to] of [[-1, 1], [0, -1], [3, 0], [0, 3], [1, 1]]) {
      act(() => { result.current.reorderTabs(from, to); });
      expect(result.current.tabs).toBe(before);
    }
  });

  it("ferme l'onglet actif et conserve les opérations PTY runtime", async () => {
    const { result } = renderHook(() => useTerminal(GROUP_KEY, DEFAULT_CWD, ready()));
    await waitFor(() => expect(result.current.persistenceStatus).toBe("healthy"));
    act(() => { result.current.addTab("/a"); });
    act(() => { result.current.addTab("/b"); });
    const firstId = result.current.tabs[0].id;
    act(() => { result.current.setPtyId(firstId, 42, "token"); });
    expect(result.current.getGroupPtyEntries(GROUP_KEY)).toEqual([{ id: 42, token: "token" }]);

    act(() => { result.current.setActiveTab(firstId); });
    act(() => { result.current.closeTab(firstId); });
    expect(result.current.tabs).toHaveLength(1);
    expect(result.current.activeTabId).toBe(result.current.tabs[0].id);
  });
});
