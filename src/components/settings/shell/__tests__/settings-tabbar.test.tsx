import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SettingsTabbar } from "../settings-tabbar";

const ITEMS = [
  { id: "plugins", label: "Plugins" },
  { id: "custom", label: "Extensions" },
  { id: "host", label: "Hôte" },
];

describe("SettingsTabbar", () => {
  afterEach(() => cleanup());

  it("marque un seul onglet actif", () => {
    render(
      <SettingsTabbar items={ITEMS} active="custom" label="Extensions" onChange={vi.fn()} />,
    );

    const selected = screen.getAllByRole("tab").filter(
      (tab) => tab.getAttribute("aria-selected") === "true",
    );

    expect(selected.map((tab) => tab.textContent)).toEqual(["Extensions"]);
  });

  it("remonte l'identifiant de l'onglet cliqué", () => {
    const onChange = vi.fn();
    render(
      <SettingsTabbar items={ITEMS} active="plugins" label="Extensions" onChange={onChange} />,
    );

    fireEvent.click(screen.getByText("Hôte"));

    expect(onChange).toHaveBeenCalledWith("host");
  });

  it("nomme la barre pour les lecteurs d'écran", () => {
    render(
      <SettingsTabbar items={ITEMS} active="plugins" label="Extensions" onChange={vi.fn()} />,
    );

    expect(screen.getByRole("tablist").getAttribute("aria-label")).toBe("Extensions");
  });
});
