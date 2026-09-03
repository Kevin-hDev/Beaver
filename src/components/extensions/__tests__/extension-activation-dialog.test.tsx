import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionActivationDialog } from "../extension-activation-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const extension: ExtensionRecord = {
  manifest: {
    id: "com.example.untrusted",
    name: "Untrusted",
    version: "1.0.0",
    beaverApi: "1",
    runtime: "node",
    access: "full",
    apiLevel: "stable",
    essential: false,
  },
  kind: "local",
  source: "/extension",
  enabled: false,
  trusted: false,
  showInChat: true,
  status: "inactive",
  contributions: { tools: [], events: [] },
};

describe("ExtensionActivationDialog", () => {
  it("requires an explicit confirmation before first activation", () => {
    const onConfirm = vi.fn();
    render(
      <ExtensionActivationDialog
        extension={extension}
        busy={false}
        errorKey={null}
        onCancel={vi.fn()}
        onConfirm={onConfirm}
      />,
    );

    expect(screen.getByRole("dialog")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", {
      name: "extensions.activation.confirm",
    }));
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it("affiche l'erreur d'activation dans la fenêtre", () => {
    render(
      <ExtensionActivationDialog
        extension={extension}
        busy={false}
        errorKey="extensions.errors.codes.extensions_fingerprint_changed"
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />,
    );

    expect(screen.getByRole("alert"))
      .toHaveTextContent("extensions.errors.codes.extensions_fingerprint_changed");
    expect(screen.getByRole("button", { name: "extensions.activation.confirm" }))
      .toBeEnabled();
  });

  it("exige un consentement distinct pour un module UI avancé", () => {
    const onConfirm = vi.fn();
    render(
      <ExtensionActivationDialog
        extension={{
          ...extension,
          manifest: {
            ...extension.manifest,
            apiLevel: "advanced",
            ui: { apiVersion: "1", mode: "advanced", entry: "ui.tsx" },
          },
        }}
        busy={false}
        errorKey={null}
        onCancel={vi.fn()}
        onConfirm={onConfirm}
      />,
    );

    const confirm = screen.getByRole("button", {
      name: "extensions.activation.confirm",
    });
    expect(screen.getByText("extensions.activation.advancedDescription"))
      .toBeInTheDocument();
    expect(confirm).toBeDisabled();

    fireEvent.click(screen.getByRole("checkbox", {
      name: "extensions.activation.advancedConfirmation",
    }));
    expect(confirm).toBeEnabled();
    fireEvent.click(confirm);
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it("place le focus et se ferme avec Échap", () => {
    const onCancel = vi.fn();
    render(
      <ExtensionActivationDialog
        extension={extension}
        busy={false}
        errorKey={null}
        onCancel={onCancel}
        onConfirm={vi.fn()}
      />,
    );

    expect(screen.getByText("extensions.actions.cancel")).toHaveFocus();
    expect(document.body.querySelector(".wk-dialog-overlay"))
      .toHaveAttribute("role", "presentation");
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledOnce();
  });
});
