import { describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { CloneGitArchiveDialog } from "../clone-git-archive-dialog";

vi.mock("@/hooks/use-keyboard", () => ({ useKeyboard: () => {} }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, string>) => {
      if (key === "agentLocal.clone.gitArchiveDescription") return `Linked ${opts?.branch}`;
      return {
        "agentLocal.clone.gitArchiveTitle": "Archive clone?",
        "agentLocal.clone.gitArchiveKeep": "Yes",
        "agentLocal.clone.gitArchiveCancel": "Cancel",
        "agentLocal.clone.gitArchiveCleanup": "Clean and archive",
      }[key] ?? key;
    },
  }),
}));

describe("CloneGitArchiveDialog", () => {
  it("affiche la branche liée et déclenche les actions", () => {
    const onArchive = vi.fn();
    const onCancel = vi.fn();
    const onCleanupArchive = vi.fn();
    render(
      <CloneGitArchiveDialog
        branchName="clone-11111111"
        onArchive={onArchive}
        onCancel={onCancel}
        onCleanupArchive={onCleanupArchive}
      />,
    );

    expect(screen.getByText("Linked clone-11111111")).not.toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Yes" }));
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    fireEvent.click(screen.getByRole("button", { name: "Clean and archive" }));

    expect(onArchive).toHaveBeenCalledTimes(1);
    expect(onCancel).toHaveBeenCalledTimes(1);
    expect(onCleanupArchive).toHaveBeenCalledTimes(1);
  });
});
