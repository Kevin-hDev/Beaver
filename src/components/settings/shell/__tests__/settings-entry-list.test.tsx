import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SettingsEntryList } from "../settings-entry-list";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("SettingsEntryList", () => {
  afterEach(() => cleanup());

  it("rend une entrée cliquable par élément configuré", () => {
    const onSelect = vi.fn();
    render(
      <SettingsEntryList
        entries={[
          { id: "openai", label: "OpenAI" },
          { id: "mistral", label: "Mistral" },
        ]}
        emptyMessage="vide"
        onSelect={onSelect}
      />,
    );

    expect(screen.getAllByRole("button")).toHaveLength(2);
    fireEvent.click(screen.getByText("Mistral"));

    expect(onSelect).toHaveBeenCalledWith("mistral");
  });

  it("affiche le message d'attente plutôt qu'une bulle vide", () => {
    render(
      <SettingsEntryList entries={[]} emptyMessage="aucun connecteur" onSelect={vi.fn()} />,
    );

    expect(screen.getByText("aucun connecteur")).toBeTruthy();
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("nomme l'état hors service, qu'une pastille seule laisserait muet", () => {
    render(
      <SettingsEntryList
        entries={[{ id: "canva", label: "Canva", offlineLabel: "déconnecté" }]}
        emptyMessage="vide"
        onSelect={vi.fn()}
      />,
    );

    expect(screen.getByRole("button").getAttribute("aria-label")).toBe("Canva — déconnecté");
  });

  it("place le contenu libre avant le chevron", () => {
    render(
      <SettingsEntryList
        entries={[{ id: "q4", label: "q4_K_M", trailing: <span>installé</span> }]}
        emptyMessage="vide"
        onSelect={vi.fn()}
      />,
    );

    expect(screen.getByText("installé")).toBeTruthy();
  });
});
