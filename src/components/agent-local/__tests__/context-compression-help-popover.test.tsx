import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { ResolvedCompressionProfileView } from "@/types/compression-profile.generated";
import { ContextProgress } from "../context-progress";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => ({
    "agentLocal.contextUsage.title": "Contexte",
    "agentLocal.contextUsage.compression": "Compression",
    "agentLocal.contextUsage.compressionDisabled": "Compression désactivée",
    "agentLocal.contextUsage.compressionHelpTitle": "Pourquoi la compression est désactivée",
    "agentLocal.contextUsage.compressionHelp": "La compression est désactivée sous 64K car le contexte réinjecté peut saturer la fenêtre.",
    "agentLocal.contextUsage.compressionHelpDismiss": "Fermer l'explication",
  })[key] ?? key }),
}));

const unavailable: ResolvedCompressionProfileView = {
  id: "beaver",
  name: "Beaver",
  source: "global",
  profile_revision: 1,
  global_selection_revision: 1,
  context_window: 32_000,
  band: "under_64k",
  available: false,
};

describe("ContextCompressionHelpPopover", () => {
  it("empile l'explication puis ferme un seul niveau par Échap", async () => {
    const user = userEvent.setup();
    render(<ContextProgress used={1_000} max={32_000} compression={unavailable} />);
    const ring = screen.getByRole("button", { name: "Contexte" });
    await user.click(ring);
    expect(screen.getByRole("dialog", { name: "Contexte" })).toBeInTheDocument();

    const help = screen.getByRole("button", {
      name: "Pourquoi la compression est désactivée",
    });
    await user.click(help);
    expect(screen.getByRole("dialog", {
      name: "Pourquoi la compression est désactivée",
    })).toBeInTheDocument();

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", {
      name: "Pourquoi la compression est désactivée",
    })).toBeNull();
    expect(screen.getByRole("dialog", { name: "Contexte" })).toBeInTheDocument();
    expect(help).toHaveFocus();

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: "Contexte" })).toBeNull();
    expect(ring).toHaveFocus();
  });

  it("intercepte le clic extérieur et affiche les noms comme texte", async () => {
    const user = userEvent.setup();
    const destructive = vi.fn();
    const malicious = { ...unavailable, available: true, name: "<img src=x>" };
    const { rerender } = render(
      <>
        <button type="button" onClick={destructive}>Détruire</button>
        <ContextProgress used={1_000} max={32_000} compression={unavailable} />
      </>,
    );
    await user.click(screen.getByRole("button", { name: "Contexte" }));
    await user.click(screen.getByRole("button", {
      name: "Pourquoi la compression est désactivée",
    }));
    await user.click(screen.getByRole("button", { name: "Fermer l'explication" }));
    expect(destructive).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "Contexte" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Détruire" }));
    expect(destructive).not.toHaveBeenCalled();
    expect(screen.queryByRole("dialog", { name: "Contexte" })).toBeNull();

    rerender(<>
      <button type="button" onClick={destructive}>Détruire</button>
      <ContextProgress used={1_000} max={32_000} compression={malicious} />
    </>);
    await user.click(screen.getByRole("button", { name: "Contexte" }));
    expect(screen.getByText("<img src=x>")).toBeInTheDocument();
    expect(document.querySelector("img[src='x']")).toBeNull();
  });
});
