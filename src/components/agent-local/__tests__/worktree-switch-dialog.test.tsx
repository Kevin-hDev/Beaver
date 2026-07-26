import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { WorktreeSwitchDialog } from "../worktree-switch-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "switchWorktree.title": "Changement de dossier",
        "switchWorktree.description": "Crée une nouvelle session pour ce worktree.",
        "switchWorktree.cancel": "Annuler",
        "switchWorktree.newSession": "Nouvelle session",
        "switchWorktree.close": "Fermer",
      };
      return translations[key] ?? key;
    },
  }),
}));

afterEach(() => cleanup());

describe("WorktreeSwitchDialog", () => {
  it("proposes only cancel or new session", () => {
    render(
      <WorktreeSwitchDialog
        branch="feature/worktree"
        path="/tmp/worktree"
        onCancel={vi.fn()}
        onNewSession={vi.fn()}
      />,
    );

    expect(screen.getByText("Changement de dossier")).not.toBeNull();
    expect(screen.getByText("feature/worktree - /tmp/worktree")).not.toBeNull();
    expect(screen.getByText("Annuler")).not.toBeNull();
    expect(screen.getByText("Nouvelle session")).not.toBeNull();
    expect(screen.queryByText("Continuer dans cette session")).toBeNull();
  });

  it("calls the expected action buttons", () => {
    const cancel = vi.fn();
    const newSession = vi.fn();
    render(
      <WorktreeSwitchDialog
        branch=""
        path="/tmp/worktree"
        onCancel={cancel}
        onNewSession={newSession}
      />,
    );

    fireEvent.click(screen.getByText("Nouvelle session"));
    fireEvent.click(screen.getByText("Annuler"));

    expect(newSession).toHaveBeenCalledOnce();
    expect(cancel).toHaveBeenCalledOnce();
  });
});
