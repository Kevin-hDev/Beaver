import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import type { CompressionProfile } from "@/types/compression-profile.generated";
import { CompressionProfileBar } from "../compression-profile-bar";
import { compressionProfilesViewFixture } from "@/test-utils/compression-profile-fixture";

const showToast = vi.hoisted(() => vi.fn());

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: { name?: string }) => values?.name ? `${key}:${values.name}` : key,
  }),
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast }));

const profile = (id: string, name: string) => ({ id, name, revision: 1 }) as CompressionProfile;

function controller(active = "custom"): CompressionProfilesController {
  return {
    view: compressionProfilesViewFixture(
      [profile("beaver", "Beaver"), profile("custom", "Longues sessions")],
      active,
    ),
    busy: false,
    setAutomaticEnabled: vi.fn(() => Promise.resolve(true)),
    selectGlobal: vi.fn(() => Promise.resolve(true)),
    save: vi.fn(() => Promise.resolve(true)),
    create: vi.fn(() => Promise.resolve(true)),
    rename: vi.fn(() => Promise.resolve(true)),
    resetBeaver: vi.fn(() => Promise.resolve(null)),
    resetPrompts: vi.fn(() => Promise.resolve(true)),
    deleteProfile: vi.fn(() => Promise.resolve({
      view: {
        ...compressionProfilesViewFixture([profile("beaver", "Beaver")]),
        global_selection_revision: 2,
      },
      undo_token: "undo-token",
      undo_expires_in_ms: 30_000,
    })),
    undoDelete: vi.fn(() => Promise.resolve(true)),
    refresh: vi.fn(() => Promise.resolve()),
  };
}

beforeEach(() => vi.clearAllMocks());

describe("CompressionProfileBar", () => {
  it("crée une copie nommée de la vraie source avec Entrée une seule fois", async () => {
    const api = controller();
    // Test spy extracted from a structural controller, with no `this` usage.
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const create = api.create;
    render(<CompressionProfileBar controller={api} onInteractionChange={() => undefined} />);
    const newButton = screen.getByRole("button", { name: "settings.advanced.compressionNewProfile" });
    fireEvent.click(newButton);
    expect(screen.getByText(/compressionCreateFrom:Longues sessions/)).toBeInTheDocument();
    const input = screen.getByRole("textbox", { name: "settings.advanced.compressionProfileName" });
    fireEvent.change(input, { target: { value: "Petit modèle" } });
    fireEvent.keyDown(input, { key: "Enter" });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() => expect(create).toHaveBeenCalledOnce());
    expect(create).toHaveBeenCalledWith("custom", "Petit modèle");
    await waitFor(() => expect(newButton).toHaveFocus());
  });

  it("renomme avec Entrée et annule avec Échap", async () => {
    const api = controller();
    // Test spy extracted from a structural controller, with no `this` usage.
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const rename = api.rename;
    render(<CompressionProfileBar controller={api} onInteractionChange={() => undefined} />);
    fireEvent.click(screen.getByRole("button", { name: "settings.advanced.compressionRename" }));
    const input = screen.getByRole("textbox", { name: "settings.advanced.compressionProfileName" });
    fireEvent.change(input, { target: { value: "Nouveau nom" } });
    fireEvent.keyDown(window, { key: "Escape" });
    expect(rename).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "settings.advanced.compressionRename" }));
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "Nouveau nom" } });
    fireEvent.keyDown(window, { key: "Enter" });
    await waitFor(() => expect(rename).toHaveBeenCalledWith("custom", "Nouveau nom"));
  });

  it("verrouille uniquement renommer et supprimer sur Beaver", () => {
    render(<CompressionProfileBar controller={controller("beaver")} onInteractionChange={() => undefined} />);
    expect(screen.getByRole("button", { name: "settings.advanced.compressionRename" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "settings.advanced.compressionDelete" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "settings.advanced.compressionReset" })).toBeEnabled();
  });

  it("permet d'annuler la restauration des valeurs Beaver", async () => {
    const api = controller("beaver");
    api.resetBeaver = vi.fn(() => Promise.resolve({
      view: api.view!,
      undo_token: "reset-token",
      undo_expires_in_ms: 30_000,
    }));
    // Test spies extracted from a structural controller, with no `this` usage.
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const reset = api.resetBeaver;
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const undo = api.undoDelete;
    render(<CompressionProfileBar controller={api} onInteractionChange={() => undefined} />);

    fireEvent.click(screen.getByRole("button", {
      name: "settings.advanced.compressionReset",
    }));

    await waitFor(() => expect(reset).toHaveBeenCalledOnce());
    const options = showToast.mock.calls[0][3] as { action: { onClick: () => void } };
    options.action.onClick();
    expect(undo).toHaveBeenCalledWith("reset-token");
  });

  it("supprime après confirmation et expose l'annulation backend", async () => {
    const api = controller();
    // Test spies extracted from a structural controller, with no `this` usage.
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const remove = api.deleteProfile;
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const undo = api.undoDelete;
    render(<CompressionProfileBar controller={api} onInteractionChange={() => undefined} />);
    fireEvent.click(screen.getByRole("button", { name: "settings.advanced.compressionDelete" }));
    fireEvent.keyDown(window, { key: "Enter" });
    await waitFor(() => expect(remove).toHaveBeenCalledWith("custom"));
    const options = showToast.mock.calls[0][3] as { action: { onClick: () => void } };
    options.action.onClick();
    expect(undo).toHaveBeenCalledWith("undo-token");
  });
});
