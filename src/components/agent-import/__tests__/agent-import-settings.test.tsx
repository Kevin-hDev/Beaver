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
