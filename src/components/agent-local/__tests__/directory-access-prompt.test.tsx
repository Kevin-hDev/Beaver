import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DirectoryAccessPrompt } from "../directory-access-prompt";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => (key === "common.cancel" ? "Annuler" : key),
  }),
}));

describe("DirectoryAccessPrompt", () => {
  it("affiche les racines autorisées et les deux actions attendues", async () => {
    const onCancel = vi.fn();
    const onSettings = vi.fn();
    render(
      <DirectoryAccessPrompt
        allowedPaths={["/project/allowed"]}
        onCancel={onCancel}
        onSettings={onSettings}
      />,
    );

    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("/project/allowed")).toBeInTheDocument();
    const cancel = screen.getByRole("button", { name: "Annuler" });
    await waitFor(() => expect(cancel).toHaveFocus());
    fireEvent.click(cancel);
    fireEvent.click(screen.getByRole("button", { name: "directoryAccess.settings" }));

    expect(onCancel).toHaveBeenCalledOnce();
    expect(onSettings).toHaveBeenCalledOnce();
  });

  it.each(["dark", "light"])("conserve le popover avec le thème %s", (theme) => {
    const { container } = render(
      <div data-theme={theme}>
        <DirectoryAccessPrompt
          allowedPaths={["/project/allowed"]}
          onCancel={vi.fn()}
          onSettings={vi.fn()}
        />
      </div>,
    );

    expect(container.querySelector(`[data-theme="${theme}"] .dap-root`)).toBeTruthy();
  });
});
