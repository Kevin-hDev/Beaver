import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionUiStartupState } from "@/types/extensions";
import { ExtensionUiRecoveryDialog } from "../extension-ui-recovery-dialog";
import { openExtensionAfterSafeChoice } from "../extension-ui-startup-boundary";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const pending: ExtensionUiStartupState = {
  mode: {
    kind: "pendingInterruptedUi",
    extensionId: "com.example.ui",
    stage: "mount",
    startedAt: "2026-09-03T10:00:00Z",
    attempts: 2,
  },
  bootstrapResolved: true,
  thirdPartyLoadingAllowed: false,
  showRecoveryDialog: true,
  showSafeBanner: false,
  canRetry: true,
};

describe("ExtensionUiRecoveryDialog", () => {
  it("does not navigate when the safe transition fails", async () => {
    const open = vi.fn();
    await openExtensionAfterSafeChoice("com.example.ui", () => Promise.resolve(false), open);
    expect(open).not.toHaveBeenCalled();
    await openExtensionAfterSafeChoice("com.example.ui", () => Promise.resolve(true), open);
    expect(open).toHaveBeenCalledWith("com.example.ui");
  });
  it("focuses the safe action and maps close, overlay and Escape to it", () => {
    const onSafe = vi.fn();
    render(
      <ExtensionUiRecoveryDialog
        state={pending}
        busy={false}
        onSafe={onSafe}
        onOpen={vi.fn()}
        onRetry={vi.fn()}
        onDiscard={vi.fn()}
      />,
    );
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.safe" })).toHaveFocus();
    fireEvent.keyDown(window, { key: "Escape" });
    fireEvent.click(document.body.querySelector(".extur-overlay")!);
    expect(onSafe).toHaveBeenCalledTimes(2);
  });

  it("makes every applicable choice keyboard-accessible in order", async () => {
    const user = userEvent.setup();
    const onOpen = vi.fn();
    const onRetry = vi.fn();
    const view = render(
      <ExtensionUiRecoveryDialog
        state={pending}
        busy={false}
        onSafe={vi.fn()}
        onOpen={onOpen}
        onRetry={onRetry}
        onDiscard={vi.fn()}
      />,
    );
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.safe" })).toHaveFocus();
    await user.tab();
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.open" })).toHaveFocus();
    await user.keyboard("{Enter}");
    await user.tab();
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.retry" })).toHaveFocus();
    await user.keyboard(" ");
    expect(onOpen).toHaveBeenCalledWith("com.example.ui");
    expect(onRetry).toHaveBeenCalledOnce();
    await user.tab();
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.safe" })).toHaveFocus();
    await user.tab({ shift: true });
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.retry" })).toHaveFocus();

    view.unmount();
    const onDiscard = vi.fn();
    render(
      <ExtensionUiRecoveryDialog
        state={{
          ...pending,
          mode: { kind: "safe", reason: "invalidMarker" },
          showSafeBanner: true,
          canRetry: false,
        }}
        busy={false}
        onSafe={vi.fn()}
        onOpen={vi.fn()}
        onRetry={vi.fn()}
        onDiscard={onDiscard}
      />,
    );
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.safe" })).toHaveFocus();
    await user.tab();
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.discard" })).toHaveFocus();
    await user.keyboard("{Enter}");
    expect(onDiscard).toHaveBeenCalledOnce();
  });

  it("shows only the discard path for an invalid marker", () => {
    const onDiscard = vi.fn();
    render(
      <ExtensionUiRecoveryDialog
        state={{
          ...pending,
          mode: { kind: "safe", reason: "invalidMarker" },
          showSafeBanner: true,
          canRetry: false,
        }}
        busy={false}
        onSafe={vi.fn()}
        onOpen={vi.fn()}
        onRetry={vi.fn()}
        onDiscard={onDiscard}
      />,
    );
    expect(screen.queryByRole("button", { name: "extensions.uiRecovery.retry" })).toBeNull();
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.discard" })).toBeEnabled();
  });

  it("shows only a generic error while keeping the safe choice", () => {
    render(
      <ExtensionUiRecoveryDialog
        state={pending}
        busy={false}
        error
        onSafe={vi.fn()}
        onOpen={vi.fn()}
        onRetry={vi.fn()}
        onDiscard={vi.fn()}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("extensions.uiRecovery.error");
    expect(screen.getByRole("button", { name: "extensions.uiRecovery.safe" })).toBeEnabled();
  });
});
