import { describe, it, expect, vi } from "vitest";
import { render, fireEvent, screen } from "@testing-library/react";
import { ContextMenu } from "../context-menu";

vi.mock("@/hooks/use-click-outside", () => ({ useClickOutside: () => {} }));
vi.mock("@/hooks/use-keyboard", () => ({ useKeyboard: () => {} }));

describe("menu contextuel", () => {
  it("se ferme après une commande ordinaire", () => {
    const onClose = vi.fn();
    render(
      <ContextMenu x={0} y={0} onClose={onClose} items={[{ label: "Archiver", onClick: vi.fn() }]} />,
    );

    fireEvent.click(screen.getByText("Archiver"));

    expect(onClose).toHaveBeenCalled();
  });

  /* Une commande dont le résultat s'affiche sur sa propre ligne doit survivre au
     clic : fermer le menu effacerait la confirmation au moment de la montrer. */
  it("reste ouvert après une commande qui le demande", () => {
    const onClose = vi.fn();
    render(
      <ContextMenu x={0} y={0} onClose={onClose} items={[{ label: "Copier l'ID", keepOpen: true, onClick: vi.fn() }]} />,
    );

    fireEvent.click(screen.getByText("Copier l'ID"));

    expect(onClose).not.toHaveBeenCalled();
  });
});
