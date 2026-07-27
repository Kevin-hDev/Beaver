import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { open } from "@tauri-apps/plugin-dialog";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ExtensionAddDialog } from "../extension-add-dialog";

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("ExtensionAddDialog", () => {
  beforeEach(() => {
    vi.mocked(open).mockReset().mockResolvedValue("/tmp/extension.ts");
  });

  it("affiche l’avertissement d’accès complet avant la sélection", () => {
    render(<ExtensionAddDialog onAdd={vi.fn()} onClose={vi.fn()} />);

    expect(screen.getByText("extensions.add.fullAccessWarning")).toBeInTheDocument();
  });

  it("reste ouvert et affiche une erreur générique si l’ajout échoue", async () => {
    const onClose = vi.fn();
    render(
      <ExtensionAddDialog
        onAdd={vi.fn().mockResolvedValue(false)}
        onClose={onClose}
      />,
    );

    fireEvent.click(screen.getByText("extensions.add.file"));

    await waitFor(() =>
      expect(screen.getByRole("alert")).toHaveTextContent("extensions.errors.operation"));
    expect(onClose).not.toHaveBeenCalled();
  });

  it("se ferme uniquement après un ajout réussi", async () => {
    const onClose = vi.fn();
    render(
      <ExtensionAddDialog
        onAdd={vi.fn().mockResolvedValue(true)}
        onClose={onClose}
      />,
    );

    fireEvent.click(screen.getByText("extensions.add.file"));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
  });
});
