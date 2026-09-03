import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useTabHistory } from "../use-tab-history";
import { DEFAULT_AGENT_LOCAL_NAV, DEFAULT_APP_NAV, FILE_ACCESS_SETTINGS_NAV, migrateAppNav } from "@/types/navigation";
import { CORE_NAVIGATION_AVAILABILITY } from "@/features/extension-ui/slot-navigation";
import type { NavigationAvailability } from "@/features/extension-ui/slot-navigation";

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

  it("retire les anciens états visuels globaux de la navigation", () => {
    const legacy = {
      ...DEFAULT_APP_NAV,
      agentLocal: { ...DEFAULT_AGENT_LOCAL_NAV, forecastSection: "notes" },
    } as unknown as typeof DEFAULT_APP_NAV;

    const { result } = renderHook(() => useTabHistory(legacy));

    expect(result.current.current.agentLocal).toEqual({ sessionId: null });
  });

  it("retire l'ancien onglet terminal actif sans republier sa valeur", () => {
    const legacy = {
      ...DEFAULT_APP_NAV,
      agentLocal: {
        ...DEFAULT_AGENT_LOCAL_NAV,
        terminalOpen: true,
        terminalActiveTabId: "stale",
      },
    } as unknown as typeof DEFAULT_APP_NAV;

    const migrated = migrateAppNav(legacy);

    expect("terminalActiveTabId" in migrated.agentLocal).toBe(false);
    expect("terminalOpen" in migrated.agentLocal).toBe(false);
  });

  it("migre l'ancien onglet Applications externes vers Extensions", () => {
    const legacy = {
      ...DEFAULT_APP_NAV,
      settings: { ...DEFAULT_APP_NAV.settings, extensionsSection: "external" },
    } as unknown as typeof DEFAULT_APP_NAV;

    expect(migrateAppNav(legacy).settings.extensionsSection).toBe("custom");
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

  it("ouvre directement le réglage d’accès aux fichiers", () => {
    const { result } = renderHook(() => useTabHistory(DEFAULT_APP_NAV));

    act(() => result.current.pushNav(FILE_ACCESS_SETTINGS_NAV));

    expect(result.current.current.tab).toBe("settings");
    expect(result.current.current.settings.subTab).toBe("advanced");
    expect(result.current.current.settings.advancedTarget).toBe("file-access");
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

  it("remplace une extension disparue sans détruire le reste de l'historique", () => {
    const extensionTab = "extension:acme:dashboard" as const;
    const available: NavigationAvailability = {
      ...CORE_NAVIGATION_AVAILABILITY,
      mainTabs: [...CORE_NAVIGATION_AVAILABILITY.mainTabs, extensionTab],
    };
    const view = renderHook(
      ({ availability }: { availability: NavigationAvailability }) =>
        useTabHistory(DEFAULT_APP_NAV, availability),
      { initialProps: { availability: available } },
    );

    act(() => view.result.current.pushNav({
      tab: extensionTab,
      heartbeat: { wakeupId: "kept" },
    }));
    expect(view.result.current.current.tab).toBe(extensionTab);

    view.rerender({ availability: CORE_NAVIGATION_AVAILABILITY });
    expect(view.result.current.current.tab).toBe("agent-local");
    expect(view.result.current.current.heartbeat.wakeupId).toBe("kept");

    act(() => view.result.current.goBack());
    act(() => view.result.current.goForward());
    expect(view.result.current.current.tab).toBe("agent-local");
    expect(view.result.current.current.heartbeat.wakeupId).toBe("kept");
  });
});
