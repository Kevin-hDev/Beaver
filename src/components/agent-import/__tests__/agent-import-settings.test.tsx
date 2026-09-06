import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AgentImportSettings } from "../agent-import-settings";

vi.mock("../agent-import-wizard", () => ({
  AgentImportWizard: () => <div>Assistant migration</div>,
}));

afterEach(cleanup);

describe("AgentImportSettings", () => {
  it("ferme la migration avec Échap", () => {
    render(<AgentImportSettings />);
    fireEvent.click(screen.getByText("agentImport.settings.manage"));
    expect(screen.getByRole("dialog")).toBeInTheDocument();

    fireEvent.keyDown(document, { key: "Escape" });

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  /* Le calque doit sortir de la page de Réglages. Rendu dedans, le fondu posé
     sous le titre figé force WebKit à peindre tout le sous-arbre dans un tampon
     aux dimensions du panneau : le dialogue était découpé au bord de la barre
     latérale, effacé en haut, et la molette remontait à la page derrière. */
  it("rend la migration hors du conteneur de la page", () => {
    const { container } = render(<AgentImportSettings />);
    fireEvent.click(screen.getByText("agentImport.settings.manage"));

    expect(container.querySelector(".aim-dialog-backdrop")).toBeNull();
    expect(document.body.querySelector(".aim-dialog-backdrop")).not.toBeNull();
  });

  it("ferme uniquement lors d'un clic à l'extérieur", () => {
    render(<AgentImportSettings />);
    fireEvent.click(screen.getByText("agentImport.settings.manage"));
    const dialog = screen.getByRole("dialog");

    fireEvent.mouseDown(dialog);
    expect(screen.getByRole("dialog")).toBeInTheDocument();

    fireEvent.mouseDown(dialog.parentElement as HTMLElement);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
