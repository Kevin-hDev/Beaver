import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PermissionModeSelector } from "../permission-mode-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const labels: Record<string, string> = {
        "permissionMode.chatLabel": "Chatbot",
        "permissionMode.chatDescription": "Discuter sans accès aux outils, aux fichiers ou à Internet",
        "permissionMode.manualLabel": "Demander l’autorisation",
        "permissionMode.manualDescription": "Toujours demander avant de modifier les fichiers externes et d’utiliser Internet",
        "permissionMode.autoLabel": "Accès complet",
        "permissionMode.autoDescription": "Accès non restreint à Internet et aux fichiers de votre ordinateur",
        "permissionMode.toggleHint": "Changer de mode (Shift+Tab)",
      };
      return labels[key] ?? key;
    },
  }),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("PermissionModeSelector", () => {
  it("affiche les titres et descriptions sans chiffres visibles", () => {
    render(<PermissionModeSelector mode="auto" onChange={vi.fn()} />);

    fireEvent.click(screen.getByRole("button"));

    expect(screen.getByText("Chatbot")).toBeTruthy();
    expect(screen.getByText("Demander l’autorisation")).toBeTruthy();
    expect(screen.getAllByText("Accès complet").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Discuter sans accès aux outils, aux fichiers ou à Internet")).toBeTruthy();
    expect(screen.getByText("Toujours demander avant de modifier les fichiers externes et d’utiliser Internet")).toBeTruthy();
    expect(screen.getByText("Accès non restreint à Internet et aux fichiers de votre ordinateur")).toBeTruthy();
    expect(screen.queryByText("1")).toBeNull();
    expect(screen.queryByText("2")).toBeNull();
    expect(screen.queryByText("3")).toBeNull();
  });

  it("ouvre la liste des modes dans un portail global", () => {
    const { container } = render(
      <div data-testid="host">
        <PermissionModeSelector mode="auto" onChange={vi.fn()} />
      </div>,
    );

    fireEvent.click(screen.getByRole("button"));

    expect(document.body.querySelector(".perm-mode-dropdown")).toBeTruthy();
    expect(container.querySelector(".perm-mode-dropdown")).toBeNull();
  });

  it("masque Chatbot pour une session outillée verrouillée", () => {
    render(
      <PermissionModeSelector
        mode="manual"
        availableModes={["manual", "auto"]}
        onChange={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button"));

    expect(screen.queryByText("Chatbot")).toBeNull();
    expect(screen.getAllByText("Demander l’autorisation").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Accès complet")).toBeTruthy();
  });
});
