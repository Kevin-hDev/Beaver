import { fireEvent, render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AboutSettings } from "../about-settings";
import { PathListEditor } from "../path-list-editor";

const mocks = vi.hoisted(() => ({
  openExternal: vi.fn(),
  openDirectory: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: () => Promise.resolve("0.9.6"),
  getTauriVersion: () => Promise.resolve("2.0.0"),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({ open: mocks.openExternal }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: mocks.openDirectory }));
vi.mock("@/components/ui/icons", () => ({ ArrowSquareOut: () => <span /> }));

describe("settings CSS wiring", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("uses the colocated classes for the about view", async () => {
    const { container, getByText, getByRole } = render(<AboutSettings />);

    expect(container.querySelector(".as-root")).not.toBeNull();
    expect(container.querySelectorAll(".as-info-row")).toHaveLength(3);
    expect(container.querySelector(".as-github-btn")).not.toBeNull();
    expect(container.querySelector(".as-app-logo")).toBeInstanceOf(HTMLImageElement);
    // Le nom n'est plus un texte mais un logotype dessiné : ce qui doit rester
    // vrai, c'est qu'il est là et qu'il s'annonce sous le nom de l'application.
    expect(getByRole("img", { name: "Beaver" })).toBeInTheDocument();
    expect(container.querySelector(".as-wordmark")).not.toBeNull();
    await waitFor(() => expect(getByText("0.9.6")).not.toBeNull());

    fireEvent.click(getByText("about.viewOnGithub"));
    expect(mocks.openExternal).toHaveBeenCalledWith("https://github.com/Kevin-hDev/Beaver");
  });

  it("keeps path add, remove and reset actions connected", async () => {
    mocks.openDirectory.mockResolvedValue("/tmp/project");
    const onChange = vi.fn();
    const view = render(<PathListEditor paths={["/"]} onChange={onChange} />);

    fireEvent.click(view.getByText("+ settings.advanced.addPath"));
    await waitFor(() => expect(onChange).toHaveBeenCalledWith(["/", "/tmp/project"]));

    view.rerender(<PathListEditor paths={["/", "/tmp/project"]} onChange={onChange} />);
    fireEvent.click(view.getAllByText("×")[0]);
    expect(onChange).toHaveBeenCalledWith(["/tmp/project"]);

    fireEvent.click(view.getByText("settings.advanced.resetPaths"));
    expect(onChange).toHaveBeenCalledWith([expect.any(String)]);
  });
});
