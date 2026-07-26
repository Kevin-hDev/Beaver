import { afterEach, describe, expect, it, vi, beforeEach } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { BranchConflictDialog } from "../branch-conflict-dialog";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@/hooks/use-keyboard", () => ({ useKeyboard: () => {} }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      if (key === "branches.uncommitted") {
        const count = typeof opts?.count === "number" || typeof opts?.count === "string" ? opts.count : "";
        return `Uncommitted: ${count}`;
      }
      return {
        "branches.conflictTitle": "Commit",
        "branches.conflictDescription": "Changed files",
        "branches.commitDescription": "Description",
        "branches.commitRequired": "Commit required",
        "branches.conflictCancel": "Cancel",
        "branches.conflictCommitAndSwitch": "Commit and switch",
      }[key] ?? key;
    },
  }),
}));

describe("BranchConflictDialog", () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue([
      { path: "file.txt", status: "modified", additions: 1, deletions: 0 },
    ]);
  });

  it("keeps commit description collapsed by default", async () => {
    render(
      <BranchConflictDialog
        targetBranch="main"
        dirtyCount={1}
        projectPath="/tmp/repo"
        onCancel={vi.fn()}
        onCommitAndSwitch={vi.fn()}
      />,
    );
    await waitFor(() => expect(screen.getByText("file.txt")).not.toBeNull());
    expect(screen.getByRole("button", { name: "Description" }).getAttribute("aria-expanded")).toBe("false");
  });

  it("passes the optional description when committing", () => {
    const onCommitAndSwitch = vi.fn();
    render(
      <BranchConflictDialog
        targetBranch="main"
        dirtyCount={1}
        projectPath="/tmp/repo"
        onCancel={vi.fn()}
        onCommitAndSwitch={onCommitAndSwitch}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Description" }));
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "Ma description" } });
    fireEvent.click(screen.getByRole("button", { name: "Commit and switch" }));
    expect(onCommitAndSwitch).toHaveBeenCalledWith("main", "Ma description");
  });
});
