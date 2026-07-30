import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { SettingsPanel } from "../settings-panel";

describe("SettingsPanel", () => {
  afterEach(() => cleanup());

  it("affiche le titre et l'action sur la même ligne", () => {
    const { container } = render(
      <SettingsPanel title="Connecteurs" action={<button type="button">Parcourir</button>}>
        <p>contenu</p>
      </SettingsPanel>,
    );

    const header = container.querySelector(".settings-panel-header");

    expect(header?.textContent).toBe("ConnecteursParcourir");
    expect(screen.getByText("contenu")).toBeTruthy();
  });

  it("supprime l'en-tête sur une fiche, qui porte déjà son propre titre", () => {
    const { container } = render(
      <SettingsPanel><p>fiche</p></SettingsPanel>,
    );

    expect(container.querySelector(".settings-panel-header")).toBeNull();
  });
});
