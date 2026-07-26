import { createRef } from "react";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { BrowserTabCreation } from "../browser-events";
import type { BrowserSessionState, LocalSite } from "../browser-types";
import { BrowserPanel } from "../browser-panel";

const TAB_ONE = "11111111111111111111111111111111";

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
  closeTab: (id: string) => boolean | Promise<boolean>;
  navigate: (id: string, url: string) => Promise<boolean>;
  navigationAction: (id: string, action: "back" | "forward" | "reloadOrStop") => Promise<boolean>;
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
    t: (key: string, values?: { title?: string }) => ({
      "browser.title": "Navigateur",
      "browser.tabsLabel": "Onglets du navigateur",
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

describe("BrowserPanel", () => {
  let api: SessionApi;

  beforeEach(() => {
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
      closeTab: vi.fn().mockResolvedValue(true),
      navigate: vi.fn().mockResolvedValue(true),
      navigationAction: vi.fn().mockResolvedValue(true),
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
