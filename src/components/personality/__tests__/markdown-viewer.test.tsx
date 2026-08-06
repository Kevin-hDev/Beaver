/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MarkdownViewer } from "../markdown-viewer";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

afterEach(() => {
  cleanup();
});

function afficher() {
  return render(
    <MarkdownViewer content="# Titre" fileName="AGENTS.md" onOpenEditor={() => {}} />,
  );
}

describe("en-tête du lecteur markdown de /beaver", () => {
  /* Le bouton n'a plus de libellé visible : sans nom accessible, il ne reste
     qu'un dessin muet pour qui n'utilise pas la souris. */
  it("nomme son bouton d'ouverture malgré l'absence de libellé", () => {
    afficher();

    const bouton = screen.getByRole("button", { name: "personality.open" });

    expect(bouton.textContent).toBe("");
    expect(bouton.querySelector("svg")).toBeTruthy();
  });

  it("garde le nom du fichier en titre", () => {
    afficher();

    expect(screen.getByText("AGENTS.md")).toBeTruthy();
  });
});
