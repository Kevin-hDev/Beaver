import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { CustomSelect } from "../custom-select";

afterEach(cleanup);

/* La liste sort de son conteneur par un portail : posée dedans, elle était
   coupée au bord de la carte et un masque ou un flou placé sur un ancêtre lui
   retirait le sien. Il faut donc que le clic sur une option parte quand même —
   le panneau est « dehors » pour la fermeture au clic extérieur. */

const OPTIONS = [
  { value: "fr", label: "Français" },
  { value: "en", label: "Anglais" },
];

function open() {
  const rendered = render(<CustomSelect options={OPTIONS} value="fr" onChange={vi.fn()} />);
  fireEvent.click(screen.getByRole("button"));
  return rendered;
}

describe("CustomSelect", () => {
  it("porte la liste hors du conteneur du composant", () => {
    const { container } = open();
    expect(container.querySelector(".cs-dropdown")).toBeNull();
    expect(document.body.querySelector(".cs-dropdown")).not.toBeNull();
  });

  it("garde les options directement dans la liste", () => {
    open();
    const listbox = screen.getByRole("listbox");
    expect(listbox.className).toContain("cs-menu");
    expect(screen.getAllByRole("option")).toHaveLength(OPTIONS.length);
    for (const option of screen.getAllByRole("option")) {
      expect(option.parentElement).toBe(listbox);
    }
  });

  it("laisse partir la sélection malgré la fermeture au clic extérieur", () => {
    const onChange = vi.fn();
    render(<CustomSelect options={OPTIONS} value="fr" onChange={onChange} />);
    fireEvent.click(screen.getByRole("button"));

    const anglais = screen.getByText("Anglais");
    fireEvent.mouseDown(anglais);
    fireEvent.click(anglais);

    expect(onChange).toHaveBeenCalledWith("en");
  });

  it("referme la liste sur un clic ailleurs", () => {
    open();
    fireEvent.mouseDown(document.body);
    expect(document.body.querySelector(".cs-dropdown")).toBeNull();
  });
});
