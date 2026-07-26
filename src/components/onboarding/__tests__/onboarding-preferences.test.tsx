import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { OnboardingPreferences } from "../onboarding-preferences";

describe("OnboardingPreferences", () => {
  it("réutilise les miniatures de thèmes des réglages", () => {
    const onThemeChange = vi.fn();
    const { container } = render(
      <OnboardingPreferences
        themeChoice="dark"
        onThemeChange={onThemeChange}
        onNext={vi.fn()}
      />,
    );

    expect(container.querySelectorAll(".ts-preview")).toHaveLength(7);
    fireEvent.click(screen.getByRole("button", {
      name: i18n.t("settings.emeraldNight"),
    }));
    expect(onThemeChange).toHaveBeenCalledWith("emerald-night");
  });

  it("ouvre le menu des langues au-dessus du bouton", () => {
    const { container } = render(
      <OnboardingPreferences
        themeChoice="dark"
        onThemeChange={vi.fn()}
        onNext={vi.fn()}
      />,
    );

    expect(container.querySelector(".ss-wrap")).toHaveClass("ss-above");
  });
});
