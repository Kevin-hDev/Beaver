import { fireEvent, render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BranchSelector } from "../branch-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const text: Record<string, string> = {
        "branches.deleteRiskTitle": "Supprimer cette branche ?",
        "branches.deleteRiskDescription": "Cette branche contient du travail.",
        "branches.deleteCurrentDescription": "CL-GO passera sur {{branch}} avant de supprimer.",
        "branches.deleteCurrent": "Changer de branche et supprimer",
        "branches.deletePreserve": "Fusionner puis supprimer",
        "branches.deleteDiscardUnmerged": "Supprimer sans fusionner",
        "branches.confirmDelete": "Confirmer",
      };
      if (key === "branches.deleteBranch") {
        return `Supprimer ${typeof opts?.name === "string" ? opts.name : ""}`;
      }
      if (key === "branches.deleteWorktree") {
        return `Supprimer le worktree ${typeof opts?.name === "string" ? opts.name : ""}`;
      }
      if (key === "branches.deleteUnmerged") {
        const count = typeof opts?.count === "number" || typeof opts?.count === "string" ? opts.count : "";
        return `Commits non fusionnés : ${count}`;
      }
      if (key === "branches.deleteCurrentDescription") {
        return `CL-GO passera sur ${typeof opts?.branch === "string" ? opts.branch : ""} avant de supprimer.`;
      }
      return text[key] ?? key;
    },
  }),
}));

vi.mock("@/hooks/use-keyboard", () => ({ useKeyboard: () => {} }));
vi.mock("@/hooks/use-click-outside", () => ({ useClickOutside: () => {} }));

function renderSelector(overrides: Record<string, unknown>) {
  const git = {
    branches: [
      { name: "main", is_current: true, is_remote: false, dirty_count: 0 },
      { name: "feat/login", is_current: false, is_remote: false, dirty_count: 0 },
    ],
    worktrees: [],
    currentBranch: "main",
    isGitRepo: true,
    checkout: vi.fn().mockResolvedValue({ ok: true }),
    create: vi.fn().mockResolvedValue({ ok: true }),
    previewBranchDeletion: vi.fn(),
    deleteBranch: vi.fn().mockResolvedValue({ ok: true }),
    previewWorktreeDeletion: vi.fn(),
    deleteWorktree: vi.fn().mockResolvedValue({ ok: true }),
    ...overrides,
  };
  const view = render(
    <BranchSelector
      git={git}
      locked={false}
      onConflict={vi.fn()}
      onWorktreeSelect={vi.fn()}
      onGithubAuthRequired={vi.fn()}
    />,
  );
  fireEvent.click(view.container.querySelector(".bs-btn") as HTMLElement);
  return view;
}

describe("BranchSelector deletion", () => {
  it("demande une confirmation pendant cinq secondes avant une suppression sure", async () => {
    const previewBranchDeletion = vi.fn().mockResolvedValue({
      branch: "feat/login",
      is_current: false,
      fallback_branch: "main",
      dirty_files: [],
      unmerged_commits: 0,
    });
    const deleteBranch = vi.fn().mockResolvedValue({ ok: true });
    const { getByRole } = renderSelector({ previewBranchDeletion, deleteBranch });

    fireEvent.click(getByRole("button", { name: "Supprimer feat/login" }));
    await waitFor(() => expect(getByRole("button", { name: "Confirmer" })).toBeTruthy());
    fireEvent.click(getByRole("button", { name: "Confirmer" }));

    await waitFor(() => expect(deleteBranch).toHaveBeenCalledWith("feat/login", "clean"));
  });

  it("explique les commits non fusionnes et laisse supprimer quand meme", async () => {
    const previewBranchDeletion = vi.fn().mockResolvedValue({
      branch: "feat/login",
      is_current: false,
      fallback_branch: "main",
      dirty_files: [],
      unmerged_commits: 2,
    });
    const deleteBranch = vi.fn().mockResolvedValue({ ok: true });
    const { getByRole } = renderSelector({ previewBranchDeletion, deleteBranch });

    fireEvent.click(getByRole("button", { name: "Supprimer feat/login" }));
    await waitFor(() => expect(getByRole("dialog", { name: "Supprimer cette branche ?" })).toBeTruthy());
    fireEvent.click(getByRole("button", { name: "Supprimer sans fusionner" }));

    await waitFor(() => expect(deleteBranch).toHaveBeenCalledWith("feat/login", "discard", undefined));
  });

  it("avertit avant de supprimer la branche actuellement utilisee", async () => {
    const previewBranchDeletion = vi.fn().mockResolvedValue({
      branch: "main",
      is_current: true,
      fallback_branch: "feat/login",
      dirty_files: [],
      unmerged_commits: 0,
    });
    const { getByRole, getByText } = renderSelector({ previewBranchDeletion });

    fireEvent.click(getByRole("button", { name: "Supprimer main" }));

    await waitFor(() => expect(getByRole("dialog", { name: "Supprimer cette branche ?" })).toBeTruthy());
    expect(getByText("CL-GO passera sur feat/login avant de supprimer.")).toBeTruthy();
    expect(getByRole("button", { name: "Changer de branche et supprimer" })).toBeTruthy();
  });

  it("confirme puis supprime un worktree propre", async () => {
    const previewWorktreeDeletion = vi.fn().mockResolvedValue({
      path: "/repo-worktree",
      branch: "feature-worktree",
      dirty_files: [],
    });
    const deleteWorktree = vi.fn().mockResolvedValue({ ok: true });
    const { getByRole } = renderSelector({
      worktrees: [{ path: "/repo-worktree", branch: "feature-worktree", is_current: false }],
      previewWorktreeDeletion,
      deleteWorktree,
    });

    fireEvent.click(getByRole("button", { name: "Supprimer le worktree feature-worktree" }));
    await waitFor(() => expect(getByRole("button", { name: "Confirmer" })).toBeTruthy());
    fireEvent.click(getByRole("button", { name: "Confirmer" }));

    await waitFor(() => expect(deleteWorktree).toHaveBeenCalledWith("/repo-worktree", "clean"));
  });
});
