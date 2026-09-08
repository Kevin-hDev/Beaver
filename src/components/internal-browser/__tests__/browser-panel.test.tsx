import { createRef } from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { BrowserTabCreation } from "../browser-events";
import type { BrowserSessionState, LocalSite } from "../browser-types";
import { BrowserPanel } from "../browser-panel";

afterEach(() => vi.restoreAllMocks());

const TAB_ONE = "11111111111111111111111111111111";
const TAB_TWO = "22222222222222222222222222222222";
const TAB_THREE = "33333333333333333333333333333333";

interface SessionApi {
  session: BrowserSessionState | null;
  loading: boolean;
  error: boolean;
  notice: "blockedFeature" | "engineStopped" | null;
  popup: null;
  clearPopup: () => void;
  clearNotice: () => void;
  createTab: (url?: string | null, replacement?: string | null) => Promise<BrowserTabCreation | null>;
  activateTab: (id: string) => boolean | Promise<boolean>;
  reorderTabs: (ids: string[]) => Promise<boolean>;
  closeTab: (id: string) => boolean | Promise<boolean>;
  navigate: (id: string, url: string) => Promise<boolean>;
  navigationAction: (id: string, action: "back" | "forward" | "reloadOrStop") => Promise<boolean>;
  reportError: () => void;
}

const mocks = vi.hoisted(() => ({
  useSession: vi.fn<() => SessionApi>(),
  useSites: vi.fn<() => { sites: LocalSite[]; generation: number; error: boolean }>(),
  useSurface: vi.fn(() => ({ hostRef: createRef<HTMLDivElement>() })),
}));

vi.mock("../use-browser-session", () => ({ useBrowserSession: mocks.useSession }));
vi.mock("../use-local-sites", () => ({ useLocalSites: mocks.useSites }));
vi.mock("../use-browser-surface", () => ({ useBrowserSurface: mocks.useSurface }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: { title?: string; alt?: string }) => ({
      "browser.title": "Navigateur",
      "browser.tabsLabel": "Onglets du navigateur",
      "browser.reorderTabsHint": `Déplacez avec ${values?.alt ?? "Alt"}`,
      "browser.newTab": "Nouvel onglet",
      "browser.closeTab": `Fermer ${values?.title ?? ""}`,
      "browser.addTab": "Ajouter un onglet",
      "browser.back": "Retour",
      "browser.forward": "Avancer",
      "browser.reload": "Recharger",
      "browser.stop": "Arrêter le chargement",
      "browser.addressLabel": "Adresse du navigateur",
      "browser.addressPlaceholder": "Saisir une URL",
      "browser.openAddress": "Ouvrir l’adresse",
      "browser.startTitle": "Commencez à naviguer",
      "browser.startDescription": "Saisissez une URL pour ouvrir une page",
      "browser.localSites": "Sites locaux disponibles",
      "browser.tabLimitTitle": "Limite de dix onglets atteinte",
      "browser.tabLimitDescription": `Remplacer ${values?.title ?? ""} ?`,
      "browser.replaceTab": "Remplacer",
      "browser.operationFailed": "Échec du navigateur",
      "browser.blockedFeature": "Fonction désactivée",
      "browser.loading": "Chargement du navigateur",
      "filePreview.fullscreen": "Plein écran",
      "filePreview.reduce": "Réduire",
      "common.cancel": "Annuler",
    }[key] ?? key),
  }),
}));

function blankSession(generation = 1): BrowserSessionState {
  return {
    tabs: [{
      id: TAB_ONE,
      title: "",
      url: null,
      loading: false,
      canGoBack: false,
      canGoForward: false,
      released: false,
    }],
    activeTabId: TAB_ONE,
    generation,
  };
}

function sessionWithTabs(generation = 1): BrowserSessionState {
  const base = blankSession(generation).tabs[0];
  return {
    tabs: [
      { ...base, id: TAB_ONE, title: "A", url: "https://a.example/" },
      { ...base, id: TAB_TWO, title: "B", url: "https://b.example/" },
      { ...base, id: TAB_THREE, title: "C", url: "https://c.example/" },
    ],
    activeTabId: TAB_ONE,
    generation,
  };
}

describe("BrowserPanel", () => {
  let api: SessionApi;

  beforeEach(() => {
    mocks.useSurface.mockClear();
    api = {
      session: blankSession(),
      loading: false,
      error: false,
      notice: null,
      popup: null,
      clearPopup: vi.fn(),
      clearNotice: vi.fn(),
      createTab: vi.fn().mockResolvedValue(null),
      activateTab: vi.fn().mockResolvedValue(true),
      reorderTabs: vi.fn().mockResolvedValue(true),
      closeTab: vi.fn().mockResolvedValue(true),
      navigate: vi.fn().mockResolvedValue(true),
      navigationAction: vi.fn().mockResolvedValue(true),
      reportError: vi.fn(),
    };
    mocks.useSession.mockImplementation(() => api);
    mocks.useSites.mockReturnValue({
      sites: [{
        url: "http://localhost:3000/",
        title: "Application locale",
        port: 3000,
        protocol: "http",
      }],
      generation: 1,
      error: false,
    });
  });

  it("affiche l'accueil, les localhost et valide la barre d'adresse", async () => {
    const { container } = render(
      <BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />,
    );

    expect(screen.getByText("Nouvel onglet")).toBeTruthy();
    expect(container.querySelector("img.ib-tab-icon")).toHaveAttribute("alt", "");
    expect(container.querySelector("img.ib-home-icon")).toHaveAttribute("alt", "");
    expect(screen.getByRole("button", { name: "Retour" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Avancer" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Recharger" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: /Application locale/ }));
    expect(api.navigate).toHaveBeenCalledWith(TAB_ONE, "http://localhost:3000/");

    const address = screen.getByPlaceholderText("Saisir une URL");
    fireEvent.focus(address);
    fireEvent.change(address, { target: { value: "file:///tmp/test" } });
    fireEvent.submit(screen.getByRole("form", { name: "Adresse du navigateur" }));
    expect(address).toHaveAttribute("aria-invalid", "true");

    fireEvent.change(address, { target: { value: "example.com" } });
    fireEvent.submit(screen.getByRole("form", { name: "Adresse du navigateur" }));
    await waitFor(() => expect(api.navigate).toHaveBeenCalledWith(TAB_ONE, "https://example.com/"));
  });

  it("ne remplace pas le texte en cours de saisie lors d'une mise à jour CEF", () => {
    const { rerender } = render(
      <BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />,
    );
    const address = screen.getByPlaceholderText("Saisir une URL");
    fireEvent.focus(address);
    fireEvent.change(address, { target: { value: "https://typing.example/" } });
    api.session = {
      ...blankSession(2),
      tabs: [{ ...blankSession().tabs[0], url: "https://runtime.example/", title: "Runtime" }],
    };
    rerender(<BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />);
    expect(address).toHaveValue("https://typing.example/");
  });

  it("conserve le brouillon et la surface quand un vrai geste déplace l'onglet actif", async () => {
    api.session = sessionWithTabs();
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (
      this: HTMLElement,
    ) {
      const ids = [TAB_ONE, TAB_TWO, TAB_THREE];
      const index = ids.indexOf(this.getAttribute("data-drag-id") ?? "");
      if (index >= 0) return { left: index * 100, top: 0, width: 100, height: 32 } as DOMRect;
      return { left: 0, top: 0, width: 300, height: 32 } as DOMRect;
    });
    const user = userEvent.setup();
    render(<BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />);
    const address = screen.getByPlaceholderText("Saisir une URL");
    await user.click(address);
    await user.clear(address);
    await user.type(address, "https://typing.example/");
    const activeTab = screen.getByRole("tab", { name: "A" });

    await user.pointer([{ target: activeTab, keys: "[MouseLeft>]" }]);
    expect(address).toHaveFocus();
    fireEvent.pointerMove(window, { clientX: 250, clientY: 10 });
    fireEvent.pointerUp(window);
    fireEvent.click(activeTab, { detail: 1 });

    await waitFor(() => expect(api.reorderTabs).toHaveBeenCalledWith([TAB_TWO, TAB_THREE, TAB_ONE]));
    expect(address).toHaveValue("https://typing.example/");
    expect(api.activateTab).not.toHaveBeenCalled();
    expect(api.closeTab).not.toHaveBeenCalled();
    expect(mocks.useSurface).toHaveBeenLastCalledWith(expect.objectContaining({
      conversationId: "session-test",
      tabId: TAB_ONE,
      url: "https://a.example/",
    }));
  });

  it("conserve le brouillon après l'annulation d'un geste", async () => {
    api.session = sessionWithTabs();
    const user = userEvent.setup();
    render(<BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />);
    const address = screen.getByPlaceholderText("Saisir une URL");
    await user.click(address);
    await user.clear(address);
    await user.type(address, "https://draft.example/");
    const activeTab = screen.getByRole("tab", { name: "A" });

    await user.pointer([{ target: activeTab, keys: "[MouseLeft>]" }]);
    fireEvent.pointerMove(window, { clientX: 150, clientY: 10 });
    fireEvent.keyDown(window, { key: "Escape" });
    fireEvent.pointerUp(window);
    fireEvent.click(activeTab, { detail: 1 });

    expect(address).toHaveValue("https://draft.example/");
    expect(api.reorderTabs).not.toHaveBeenCalled();
    expect(api.activateTab).not.toHaveBeenCalled();
  });

  it("signale la présence réelle de la surface CEF", () => {
    const { container, rerender } = render(
      <BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />,
    );
    expect(container.querySelector(".ib-surface")).toHaveAttribute("data-native-active", "false");

    api.session = {
      ...blankSession(2),
      tabs: [{ ...blankSession().tabs[0], url: "https://example.com/" }],
    };
    rerender(<BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />);
    expect(container.querySelector(".ib-surface")).toHaveAttribute("data-native-active", "true");
  });

  it("transmet active true puis false puis true sans démonter le panneau", () => {
    api.session = {
      ...blankSession(2),
      tabs: [{ ...blankSession().tabs[0], url: "https://example.com/" }],
    };
    const view = render(
      <BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />,
    );
    const surface = view.container.querySelector(".ib-surface");
    expect(surface).not.toBeNull();
    expect(mocks.useSurface).toHaveBeenLastCalledWith(expect.objectContaining({ active: true }));

    view.rerender(<BrowserPanel conversationId="session-test" active={false} fullscreen={false} onFullscreenChange={vi.fn()} />);
    expect(mocks.useSurface).toHaveBeenLastCalledWith(expect.objectContaining({ active: false }));

    view.rerender(<BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />);
    expect(mocks.useSurface).toHaveBeenLastCalledWith(expect.objectContaining({ active: true }));
    expect(view.container.querySelector(".ib-surface")).toBe(surface);
  });

  it("confirme avant de remplacer le plus ancien onglet inactif", async () => {
    vi.mocked(api.createTab)
      .mockResolvedValueOnce({
        status: "confirmationRequired",
        candidateId: TAB_ONE,
        candidateTitle: "Ancien onglet",
      })
      .mockResolvedValueOnce({ status: "created", session: blankSession(3) });
    render(<BrowserPanel conversationId="session-test" active fullscreen={false} onFullscreenChange={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "Ajouter un onglet" }));
    expect(await screen.findByRole("dialog")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Remplacer" }));
    await waitFor(() => expect(api.createTab).toHaveBeenLastCalledWith(null, TAB_ONE));
  });
});
