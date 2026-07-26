import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ThemeSelector } from "../theme-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "settings.emeraldNight") return "Émeraude nocturne";
      if (key === "settings.cobaltFrost") return "Cobalt givré";
      if (key === "settings.astralMist") return "Brume astrale";
      if (key === "settings.crimsonEclipse") return "Éclipse écarlate";
      return key;
    },
  }),
}));

describe("ThemeSelector", () => {
  it("affiche et sélectionne Émeraude nocturne", () => {
    const onChange = vi.fn();
    const { container } = render(<ThemeSelector value="dark" onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "Émeraude nocturne" }));

    expect(onChange).toHaveBeenCalledWith("emerald-night");
    expect(container.querySelector('[data-palette="emerald-night"]')).toHaveAttribute("data-theme", "dark");
  });

  it("affiche et sélectionne Cobalt givré comme thème clair", () => {
    const onChange = vi.fn();
    const { container } = render(<ThemeSelector value="dark" onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "Cobalt givré" }));

    expect(onChange).toHaveBeenCalledWith("cobalt-frost");
    expect(container.querySelector('[data-palette="cobalt-frost"]')).toHaveAttribute("data-theme", "light");
  });

  it("affiche et sélectionne Brume astrale comme thème sombre", () => {
    const onChange = vi.fn();
    const { container } = render(<ThemeSelector value="dark" onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "Brume astrale" }));

    expect(onChange).toHaveBeenCalledWith("astral-mist");
    expect(container.querySelector('[data-palette="astral-mist"]')).toHaveAttribute("data-theme", "dark");
  });

  it("affiche et sélectionne Éclipse écarlate comme thème sombre", () => {
    const onChange = vi.fn();
    const { container } = render(<ThemeSelector value="dark" onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: "Éclipse écarlate" }));

    expect(onChange).toHaveBeenCalledWith("crimson-eclipse");
    expect(container.querySelector('[data-palette="crimson-eclipse"]')).toHaveAttribute("data-theme", "dark");
  });
});
