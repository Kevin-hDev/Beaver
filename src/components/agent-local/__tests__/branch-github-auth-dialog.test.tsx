import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BranchGithubAuthDialog } from "../branch-github-auth-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/icons", () => ({
  X: () => <span />,
}));

describe("BranchGithubAuthDialog", () => {
  it("s'affiche dans la couche globale et conserve ses interactions", () => {
    const onCancel = vi.fn();
    const onConnect = vi.fn();
    const { container, getByRole } = render(
      <div className="ssb-popover">
        <BranchGithubAuthDialog
          state="idle"
          onCancel={onCancel}
          onConnect={onConnect}
        />
      </div>,
    );

    const dialog = getByRole("dialog");
    expect(container.querySelector(".bcd-dialog")).toBeNull();
    expect(dialog.closest(".ssb-popover")).toBeNull();

    fireEvent.mouseDown(dialog);
    fireEvent.click(getByRole("button", { name: "branches.githubAuthConnect" }));

    expect(onCancel).not.toHaveBeenCalled();
    expect(onConnect).toHaveBeenCalledOnce();
  });
});
