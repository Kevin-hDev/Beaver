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
    render(
      <ExtensionAddDialog
        onAdd={vi.fn()}
        onInstall={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    expect(screen.getByText("extensions.add.fullAccessWarning")).toBeInTheDocument();
  });

  it("reste ouvert et affiche une erreur générique si l’ajout échoue", async () => {
    const onClose = vi.fn();
    render(
      <ExtensionAddDialog
        onAdd={vi.fn().mockResolvedValue(false)}
        onInstall={vi.fn()}
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
        onInstall={vi.fn()}
        onClose={onClose}
      />,
    );

    fireEvent.click(screen.getByText("extensions.add.file"));
    await waitFor(() => expect(onClose).toHaveBeenCalledTimes(1));
  });

  it.each([
    ["git", "extensions.add.git", "extensions.add.gitLabel", "https://github.com/example/ext.git"],
    ["npm", "extensions.add.npm", "extensions.add.npmLabel", "@example/ext@latest"],
  ] as const)("installe une source %s saisie explicitement", async (
    source,
    button,
    label,
    locator,
  ) => {
    const onInstall = vi.fn().mockResolvedValue(true);
    render(
      <ExtensionAddDialog
        onAdd={vi.fn()}
        onInstall={onInstall}
        onClose={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByText(button));
    fireEvent.change(screen.getByLabelText(label), { target: { value: locator } });
    fireEvent.click(screen.getByText("extensions.add.install"));

    await waitFor(() => expect(onInstall).toHaveBeenCalledWith(source, locator));
  });
});
