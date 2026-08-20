import { render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { describe, expect, it, vi } from "vitest";
import { ChatOverlays } from "../chat-overlays";

vi.mock("../file-preview", () => ({ FilePreview: () => <div data-testid="file-preview" /> }));
vi.mock("../switch-model-dialog", () => ({ SwitchModelDialog: () => <div data-testid="model-switch" /> }));
vi.mock("../worktree-switch-dialog", () => ({ WorktreeSwitchDialog: () => <div data-testid="worktree-switch" /> }));
vi.mock("../clone-session-dialog", () => ({ CloneSessionDialog: () => <div data-testid="clone-session" /> }));

const props = {
  preview: { name: "report", path: "/project/report.md" },
  currentModel: "model",
  pendingSwitch: { model: "next-model", provider: "provider" },
  pendingWorktreeSwitch: { branch: "feature", path: "/project" },
  pendingClone: { canSummarize: true, error: null },
  cloneBusy: false,
  onClosePreview: vi.fn(), onCancelSwitch: vi.fn(), onCancelWorktreeSwitch: vi.fn(),
  onCancelClone: vi.fn(), onAbortClone: vi.fn(), onSubmitClone: vi.fn(),
  onNewSession: vi.fn(), onContinue: vi.fn(), onNewWorktreeSession: vi.fn(),
} as unknown as ComponentProps<typeof ChatOverlays>;

describe("ChatOverlays child read-only mode", () => {
  it("hides inherited write dialogs but keeps file preview visible", () => {
    const { rerender } = render(<ChatOverlays {...props} readOnly />);

    expect(screen.getByTestId("file-preview")).toBeInTheDocument();
    expect(screen.queryByTestId("model-switch")).not.toBeInTheDocument();
    expect(screen.queryByTestId("worktree-switch")).not.toBeInTheDocument();
    expect(screen.queryByTestId("clone-session")).not.toBeInTheDocument();

    rerender(<ChatOverlays {...props} readOnly={false} />);

    expect(screen.getByTestId("model-switch")).toBeInTheDocument();
    expect(screen.getByTestId("worktree-switch")).toBeInTheDocument();
    expect(screen.getByTestId("clone-session")).toBeInTheDocument();
  });
});
