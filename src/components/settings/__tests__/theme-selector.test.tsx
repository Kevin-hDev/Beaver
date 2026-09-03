import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ExtensionThemeCatalog } from "@/features/extension-ui/themes/theme-catalog";
import { ThemeSelector } from "../theme-selector";

const themeCatalog = vi.hoisted<{ current: ExtensionThemeCatalog }>(() => ({
  current: { ready: false, entries: [], byChoice: new Map() },
}));

vi.mock("@/features/extension-ui/themes/theme-context", () => ({
  useThemeCatalog: () => themeCatalog.current,
}));

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
  beforeEach(() => {
    themeCatalog.current = { ready: false, entries: [], byChoice: new Map() };
  });

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

  it("affiche la source et sélectionne au clavier un thème d'extension", async () => {
    const user = userEvent.setup();
    const entry = {
      choice: "extension:com.example.night" as const,
      paletteId: "com.example.night",
      extensionId: "com.example",
      sourceName: "Example Themes",
      label: "Night",
      colorScheme: "dark" as const,
      tokens: { "--void": "#010203" },
    };
    themeCatalog.current = {
      ready: true,
      entries: [entry],
      byChoice: new Map([[entry.choice, entry]]),
    };
    const onChange = vi.fn();
    const { container } = render(<ThemeSelector value="dark" onChange={onChange} />);
    const button = screen.getByRole("button", { name: "Night" });

    button.focus();
    await user.keyboard("{Enter}");

    expect(onChange).toHaveBeenCalledWith(entry.choice);
    expect(screen.getByText("Example Themes")).toBeVisible();
    const preview = container.querySelector('[data-palette="com.example.night"]');
    expect(preview).toHaveAttribute("data-theme", "dark");
    expect((preview as HTMLElement).style.getPropertyValue("--void")).toBe("#010203");
  });
});
