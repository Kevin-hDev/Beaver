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
  return {
    tabs: [{
      id: tabId,
      title: url ? "Exemple" : "",
      url,
      loading: false,
      canGoBack: false,
      canGoForward: false,
      released: false,
    }],
    activeTabId: tabId,
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
});
