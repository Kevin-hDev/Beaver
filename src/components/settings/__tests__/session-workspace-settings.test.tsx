import { fireEvent, render, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SessionWorkspaceSettings } from "../session-workspace-settings";

const mocks = vi.hoisted(() => ({
  openDirectory: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: mocks.openDirectory }));

describe("SessionWorkspaceSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("selects and resets the custom outputs directory", async () => {
    mocks.openDirectory.mockResolvedValue("/tmp/deliverables");
    const onChange = vi.fn();
    const { getByText } = render(
      <SessionWorkspaceSettings
        outputsDirectory="/saved/outputs"
        onOutputsDirectoryChange={onChange}
      />,
    );

    expect(getByText("/saved/outputs")).not.toBeNull();
    fireEvent.click(getByText("settings.advanced.outputsDirectoryChoose"));
    await waitFor(() => expect(onChange).toHaveBeenCalledWith("/tmp/deliverables"));

    fireEvent.click(getByText("settings.advanced.outputsDirectoryReset"));
    expect(onChange).toHaveBeenCalledWith("");
  });

  it("opens Beaver data through the dedicated backend command", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const { getByText } = render(
      <SessionWorkspaceSettings outputsDirectory="" onOutputsDirectoryChange={vi.fn()} />,
    );

    fireEvent.click(getByText("settings.advanced.dataFolderOpen"));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("open_app_data_folder"));
  });
});
