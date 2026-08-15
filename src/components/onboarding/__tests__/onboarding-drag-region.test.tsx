import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OnboardingScreen } from "../onboarding-screen";

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

describe("OnboardingScreen", () => {
  it("expose une bande de déplacement de la fenêtre", () => {
    const { container } = renderScreen();

    expect(container.querySelector(".ob-drag-region")).not.toBeNull();
  });

  it("la place avant les diapositives, jamais par-dessus leurs boutons", () => {
    const { container } = renderScreen();

    const shell = container.querySelector(".ob-shell");
    expect(shell?.firstElementChild?.classList.contains("ob-drag-region")).toBe(true);
    expect(shell?.querySelector(".ob-track")).not.toBeNull();
  });
});
