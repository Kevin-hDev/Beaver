import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { SettingsDialog } from "../settings-dialog";

describe("SettingsDialog", () => {
  it("garde le clavier dans la fenêtre, ferme avec Échap et rend le focus", async () => {
    const user = userEvent.setup();
    const close = vi.fn();
    const trigger = document.createElement("button");
    document.body.append(trigger);
    trigger.focus();
    const view = render(
      <SettingsDialog title="Mode vocal" closeLabel="Fermer" onClose={close}>
        <button type="button">Parler</button>
      </SettingsDialog>,
    );
    const closeButton = screen.getAllByRole("button", { name: "Fermer" })
      .find((button) => button.tabIndex !== -1);
    if (!closeButton) throw new Error("missing_close_button");
    expect(screen.getByRole("dialog", { name: "Mode vocal" })).toBeVisible();
    expect(closeButton).toHaveFocus();
    await user.tab({ shift: true });
    expect(screen.getByRole("button", { name: "Parler" })).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(close).toHaveBeenCalledOnce();
    view.unmount();
    expect(trigger).toHaveFocus();
    trigger.remove();
  });

  it("laisse le dialogue enfant traiter Échap", async () => {
    const close = vi.fn();
    render(<SettingsDialog title="Mode vocal" onClose={close} childDialogActive>
      <button type="button">Enfant</button>
    </SettingsDialog>);
    await userEvent.setup().keyboard("{Escape}");
    expect(close).not.toHaveBeenCalled();
  });
});
