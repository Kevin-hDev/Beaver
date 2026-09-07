import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  BROWSER_BLOCKED_FEATURE_EVENT,
  BROWSER_POPUP_EVENT,
  BROWSER_SESSION_EVENT,
} from "../browser-events";
import { useBrowserSession } from "../use-browser-session";

const TAB_ONE = "11111111111111111111111111111111";
const TAB_TWO = "22222222222222222222222222222222";

function session(generation: number, tabId = TAB_ONE, url: string | null = null) {
  return sessionWithTabs(generation, [tabId], tabId, url);
}

function sessionWithTabs(
  generation: number,
  tabIds: string[],
  activeTabId = tabIds[0],
  url: string | null = null,
) {
  return {
    tabs: tabIds.map((id) => ({
      id,
      title: url && id === tabIds[0] ? "Exemple" : "",
      url: id === tabIds[0] ? url : null,
      loading: false,
      canGoBack: false,
      canGoForward: false,
      released: false,
    })),
    activeTabId,
    generation,
  };
}

describe("useBrowserSession", () => {
  const handlers = new Map<string, (event: { payload: unknown }) => void>();

  beforeEach(() => {
    handlers.clear();
    vi.mocked(listen).mockImplementation((event, callback) => {
      handlers.set(event, callback as (value: { payload: unknown }) => void);
      return Promise.resolve(() => {});
    });
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session") return Promise.resolve(session(2));
      return Promise.reject(new Error("unexpected command"));
    });
  });

  it("ouvre la session chiffrée de la conversation et ignore les événements anciens", async () => {
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.session?.generation).toBe(2);

    act(() => handlers.get(BROWSER_SESSION_EVENT)?.({
      payload: { eventVersion: 1, conversationId: "session-test", session: session(1) },
    }));
    expect(result.current.session?.generation).toBe(2);

    act(() => handlers.get(BROWSER_SESSION_EVENT)?.({
      payload: { eventVersion: 1, conversationId: "session-test", session: session(3) },
    }));
    expect(result.current.session?.generation).toBe(3);

    act(() => handlers.get(BROWSER_BLOCKED_FEATURE_EVENT)?.({
      payload: {
        eventVersion: 1,
        generation: 8,
        conversationId: "session-test",
        tabId: TAB_ONE,
      },
    }));
    expect(result.current.notice).toBe("blockedFeature");
  });

  it("demande une confirmation au onzième onglet sans navigation prématurée", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session") return Promise.resolve(session(2));
      if (command === "browser_create_tab") {
        return Promise.resolve({
          status: "confirmationRequired",
          candidateId: TAB_ONE,
          candidateTitle: "Ancien onglet",
        });
      }
      return Promise.reject(new Error("unexpected command"));
    });
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));

    const outcome: { value: Awaited<ReturnType<typeof result.current.createTab>> } = { value: null };
    await act(async () => { outcome.value = await result.current.createTab("https://example.com"); });

    expect(outcome.value?.status).toBe("confirmationRequired");
    expect(invoke).not.toHaveBeenCalledWith("browser_navigate", expect.anything());
  });

  it("ouvre une nouvelle fenêtre CEF comme un onglet interne", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session") return Promise.resolve(session(2));
      if (command === "browser_create_tab") {
        return Promise.resolve({ status: "created", session: session(3, TAB_TWO) });
      }
      if (command === "browser_navigate") {
        return Promise.resolve(session(4, TAB_TWO, "https://example.com/popup"));
      }
      return Promise.reject(new Error("unexpected command"));
    });
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));
    act(() => handlers.get(BROWSER_POPUP_EVENT)?.({
      payload: {
        eventVersion: 1,
        generation: 7,
        conversationId: "session-test",
        sourceTabId: TAB_ONE,
        url: "https://example.com/popup",
      },
    }));
    expect(result.current.popup?.generation).toBe(7);

    await act(async () => {
      await result.current.createTab(result.current.popup?.url ?? null);
      result.current.clearPopup();
    });
    expect(result.current.session?.activeTabId).toBe(TAB_TWO);
    expect(result.current.session?.generation).toBe(4);
  });

  it("enregistre une permutation valide sans naviguer", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session") {
        return Promise.resolve(sessionWithTabs(2, [TAB_ONE, TAB_TWO]));
      }
      if (command === "browser_reorder_tabs") {
        return Promise.resolve(sessionWithTabs(3, [TAB_TWO, TAB_ONE], TAB_ONE));
      }
      return Promise.reject(new Error("unexpected command"));
    });
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(invoke).mockClear();

    let outcome = false;
    await act(async () => { outcome = await result.current.reorderTabs([TAB_TWO, TAB_ONE]); });

    expect(outcome).toBe(true);
    expect(invoke).toHaveBeenCalledWith("browser_reorder_tabs", {
      conversationId: "session-test",
      tabIds: [TAB_TWO, TAB_ONE],
    });
    expect(invoke).not.toHaveBeenCalledWith("browser_navigate", expect.anything());
    expect(result.current.session?.tabs.map((tab) => tab.id)).toEqual([TAB_TWO, TAB_ONE]);
  });

  it("refuse localement une permutation hors limite ou invalide sans IPC", async () => {
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));
    vi.mocked(invoke).mockClear();

    let invalidOutcome = true;
    let oversizedOutcome = true;
    await act(async () => {
      invalidOutcome = await result.current.reorderTabs(["invalid"]);
      oversizedOutcome = await result.current.reorderTabs(
        Array.from({ length: 11 }, (_, index) => `${index + 1}`.padStart(32, "0")),
      );
    });

    expect(invalidOutcome).toBe(false);
    expect(oversizedOutcome).toBe(false);
    expect(result.current.error).toBe(true);
    expect(invoke).not.toHaveBeenCalled();
  });

  it("accepte l’ordre identique sans remplacer l’état de même génération", async () => {
    const current = sessionWithTabs(2, [TAB_ONE, TAB_TWO], TAB_TWO);
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session" || command === "browser_reorder_tabs") {
        return Promise.resolve(current);
      }
      return Promise.reject(new Error("unexpected command"));
    });
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));

    let outcome = false;
    await act(async () => { outcome = await result.current.reorderTabs([TAB_ONE, TAB_TWO]); });

    expect(outcome).toBe(true);
    expect(result.current.session).toEqual(current);
  });

  it("resynchronise puis signale une permutation refusée", async () => {
    let openCount = 0;
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session") {
        openCount += 1;
        return Promise.resolve(sessionWithTabs(openCount === 1 ? 2 : 4, [TAB_ONE, TAB_TWO]));
      }
      if (command === "browser_reorder_tabs") return Promise.reject(new Error("rejected"));
      return Promise.reject(new Error("unexpected command"));
    });
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));

    let outcome = true;
    await act(async () => { outcome = await result.current.reorderTabs([TAB_TWO, TAB_ONE]); });

    expect(outcome).toBe(false);
    expect(result.current.error).toBe(true);
    expect(result.current.session?.generation).toBe(4);
  });

  it("signale une permutation refusée lorsque la resynchronisation échoue aussi", async () => {
    let opened = false;
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session" && !opened) {
        opened = true;
        return Promise.resolve(sessionWithTabs(2, [TAB_ONE, TAB_TWO]));
      }
      return Promise.reject(new Error("rejected"));
    });
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));

    let outcome = true;
    await act(async () => { outcome = await result.current.reorderTabs([TAB_TWO, TAB_ONE]); });

    expect(outcome).toBe(false);
    expect(result.current.error).toBe(true);
    expect(result.current.session?.generation).toBe(2);
  });

  it("conserve un événement CEF plus récent que la réponse de permutation", async () => {
    let resolveReorder: (value: ReturnType<typeof sessionWithTabs>) => void = () => {};
    const reorder = new Promise<ReturnType<typeof sessionWithTabs>>((resolve) => {
      resolveReorder = resolve;
    });
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session") {
        return Promise.resolve(sessionWithTabs(2, [TAB_ONE, TAB_TWO]));
      }
      if (command === "browser_reorder_tabs") return reorder;
      return Promise.reject(new Error("unexpected command"));
    });
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));

    let outcome = Promise.resolve(false);
    act(() => { outcome = result.current.reorderTabs([TAB_TWO, TAB_ONE]); });
    act(() => handlers.get(BROWSER_SESSION_EVENT)?.({
      payload: {
        eventVersion: 1,
        conversationId: "session-test",
        session: sessionWithTabs(4, [TAB_ONE, TAB_TWO]),
      },
    }));
    await act(async () => {
      resolveReorder(sessionWithTabs(3, [TAB_TWO, TAB_ONE]));
      await outcome;
    });

    await expect(outcome).resolves.toBe(true);
    expect(result.current.session?.generation).toBe(4);
  });

  it("ignore une réponse de permutation de A après l’ouverture de B", async () => {
    let resolveReorder: (value: ReturnType<typeof sessionWithTabs>) => void = () => {};
    const reorder = new Promise<ReturnType<typeof sessionWithTabs>>((resolve) => {
      resolveReorder = resolve;
    });
    let openCount = 0;
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session") {
        openCount += 1;
        return Promise.resolve(
          openCount === 1
            ? sessionWithTabs(2, [TAB_ONE])
            : sessionWithTabs(4, [TAB_TWO]),
        );
      }
      if (command === "browser_reorder_tabs") return reorder;
      return Promise.reject(new Error("unexpected command"));
    });
    const { result, rerender } = renderHook(
      ({ conversationId }) => useBrowserSession(conversationId, true),
      { initialProps: { conversationId: "session-a" } },
    );
    await waitFor(() => expect(result.current.session?.activeTabId).toBe(TAB_ONE));

    let outcome = Promise.resolve(false);
    act(() => { outcome = result.current.reorderTabs([TAB_TWO, TAB_ONE]); });
    rerender({ conversationId: "session-b" });
    await waitFor(() => expect(result.current.session?.activeTabId).toBe(TAB_TWO));
    await act(async () => {
      resolveReorder(sessionWithTabs(3, [TAB_TWO, TAB_ONE]));
      await outcome;
    });

    await expect(outcome).resolves.toBe(true);
    expect(result.current.session?.activeTabId).toBe(TAB_TWO);
    expect(result.current.session?.generation).toBe(4);
    expect(result.current.error).toBe(false);
  });

  it("ignore le refus et la relecture tardive de A après l’ouverture de B", async () => {
    let rejectReorder: (reason?: unknown) => void = () => {};
    const reorder = new Promise<ReturnType<typeof sessionWithTabs>>((_, reject) => {
      rejectReorder = reject;
    });
    let openCount = 0;
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "browser_open_session") {
        openCount += 1;
        return Promise.resolve(
          openCount === 1 || openCount === 3
            ? sessionWithTabs(openCount === 1 ? 2 : 3, [TAB_ONE])
            : sessionWithTabs(4, [TAB_TWO]),
        );
      }
      if (command === "browser_reorder_tabs") return reorder;
      return Promise.reject(new Error("unexpected command"));
    });
    const { result, rerender } = renderHook(
      ({ conversationId }) => useBrowserSession(conversationId, true),
      { initialProps: { conversationId: "session-a" } },
    );
    await waitFor(() => expect(result.current.session?.activeTabId).toBe(TAB_ONE));

    let outcome = Promise.resolve(true);
    act(() => { outcome = result.current.reorderTabs([TAB_TWO, TAB_ONE]); });
    rerender({ conversationId: "session-b" });
    await waitFor(() => expect(result.current.session?.activeTabId).toBe(TAB_TWO));
    await act(async () => {
      rejectReorder(new Error("rejected"));
      await outcome;
    });

    await expect(outcome).resolves.toBe(false);
    expect(result.current.session?.activeTabId).toBe(TAB_TWO);
    expect(result.current.session?.generation).toBe(4);
    expect(result.current.error).toBe(false);
  });

  it("réexpose le signal d’erreur générique", async () => {
    const { result } = renderHook(() => useBrowserSession("session-test", true));
    await waitFor(() => expect(result.current.loading).toBe(false));

    act(() => result.current.reportError());

    expect(result.current.error).toBe(true);
  });
});
