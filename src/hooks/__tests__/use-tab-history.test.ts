import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useTabHistory } from "../use-tab-history";
import { DEFAULT_APP_NAV } from "@/types/navigation";

describe("useTabHistory", () => {
  it("migre l'ancien onglet api-keys vers Providers", () => {
    const legacy = {
      ...DEFAULT_APP_NAV,
      settings: { ...DEFAULT_APP_NAV.settings, subTab: "api-keys" },
    } as unknown as typeof DEFAULT_APP_NAV;

    const { result } = renderHook(() => useTabHistory(legacy));

    expect(result.current.current.settings.subTab).toBe("providers");
    expect(result.current.current.settings.providersSubTab).toBe("api");
  });

  it("migre les anciens onglets Forecast déplacés vers la vue principale", () => {
    const legacy = {
      ...DEFAULT_APP_NAV,
      agentLocal: { ...DEFAULT_APP_NAV.agentLocal, forecastSection: "notes" },
    } as unknown as typeof DEFAULT_APP_NAV;

    const { result } = renderHook(() => useTabHistory(legacy));

    expect(result.current.current.agentLocal.forecastSection).toBe("view");
  });

  it("ignore les push identiques", () => {
    const { result } = renderHook(() => useTabHistory(DEFAULT_APP_NAV));

    act(() => result.current.pushNav({ tab: "agent-local" }));

    expect(result.current.canGoBack).toBe(false);
    expect(result.current.current).toEqual(DEFAULT_APP_NAV);
  });

  it("restaure exactement retour puis suivant", () => {
    const { result } = renderHook(() => useTabHistory(DEFAULT_APP_NAV));

    act(() => result.current.pushNav({ tab: "settings" }));
    act(() => result.current.pushNav({ settings: { subTab: "providers" } }));

    expect(result.current.current.settings.subTab).toBe("providers");
    act(() => result.current.goBack());
    expect(result.current.current.tab).toBe("settings");
    expect(result.current.current.settings.subTab).toBe("general");

    act(() => result.current.goForward());
    expect(result.current.current.tab).toBe("settings");
    expect(result.current.current.settings.subTab).toBe("providers");
  });

  it("replaceNav ne cree pas d'entree historique", () => {
    const { result } = renderHook(() => useTabHistory(DEFAULT_APP_NAV));

    act(() => result.current.replaceNav({ settings: { apiKeyProviderId: "openai" } }));

    expect(result.current.current.settings.apiKeyProviderId).toBe("openai");
    expect(result.current.canGoBack).toBe(false);
  });

  it("remplace les vues a kind au lieu de garder les anciens champs", () => {
    const { result } = renderHook(() => useTabHistory(DEFAULT_APP_NAV));

    act(() => result.current.pushNav({
      settings: { llmView: { kind: "detail", modelKey: "gpt-x", parent: { kind: "idle", showFamilies: true } } },
    }));
    act(() => result.current.pushNav({ settings: { llmView: { kind: "search", query: "gpt" } } }));

    expect(result.current.current.settings.llmView).toEqual({ kind: "search", query: "gpt" });
  });

  it("un restore suivi du meme push garde le forward", () => {
    const { result } = renderHook(() => useTabHistory(DEFAULT_APP_NAV));

    act(() => result.current.pushNav({ agentLocal: { sessionId: "s1" } }));
    act(() => result.current.goBack());
    act(() => result.current.pushNav({ agentLocal: { sessionId: null } }));

    expect(result.current.canGoForward).toBe(true);
    act(() => result.current.goForward());
    expect(result.current.current.agentLocal.sessionId).toBe("s1");
  });
});
