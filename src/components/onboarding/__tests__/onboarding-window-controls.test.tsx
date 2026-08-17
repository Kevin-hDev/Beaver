import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OnboardingScreen } from "../onboarding-screen";

/* Sur Linux et Windows, l'application retire les décorations natives au
   démarrage et dessine ses propres boutons de fenêtre. Ils ne vivaient que dans
   la coquille de l'application : pendant l'accueil, la fenêtre ne pouvait être
   ni fermée, ni réduite, ni agrandie. */

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "fr", changeLanguage: vi.fn() },
  }),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    startDragging: vi.fn().mockResolvedValue(undefined),
    isMaximized: vi.fn().mockResolvedValue(false),
    maximize: vi.fn().mockResolvedValue(undefined),
    unmaximize: vi.fn().mockResolvedValue(undefined),
    minimize: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
  }),
}));

afterEach(cleanup);

function renderScreen() {
  return render(
    <OnboardingScreen
      themeChoice="dark"
      onThemeChange={vi.fn()}
      showOllamaStep={false}
      onCompleteOnboarding={vi.fn()}
      onCompleteOllama={vi.fn()}
      onSkipOllama={vi.fn()}
    />,
  );
}

describe("boutons de fenêtre de l'accueil", () => {
  it("expose fermer, réduire et agrandir", () => {
    const { container } = renderScreen();

    expect(container.querySelector(".wc-btn--close")).not.toBeNull();
    expect(container.querySelector(".wc-btn--minimize")).not.toBeNull();
    expect(container.querySelector(".wc-btn--maximize")).not.toBeNull();
  });

  /* Le même conteneur que le splash et que l'installation d'Ollama : c'est lui
     qui porte le passage au-dessus du calque de démarrage. */
  it("les place dans le conteneur des écrans de démarrage", () => {
    const { container } = renderScreen();

    const host = container.querySelector(".startup-window-controls");
    expect(host).not.toBeNull();
    expect(host?.querySelector(".window-controls")).not.toBeNull();
  });
});
