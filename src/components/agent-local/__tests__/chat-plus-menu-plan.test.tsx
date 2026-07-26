import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ChatPlusMenu } from "../chat-plus-menu";

const connectorState = vi.hoisted(() => ({
  configured: [] as Array<{
    id: string;
    display_name: string;
    status: string;
    enabled_in_chat: boolean;
  }>,
  toggleChatEnabled: vi.fn(),
}));

vi.mock("@/hooks/use-connectors", () => ({
  useConnectors: () => connectorState,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "chatMenu.addFile") return "Add file";
      if (key === "chatMenu.planMode") return "Plan mode";
      if (key === "chatMenu.planModeDesc") return "Prepare a plan";
      if (key === "chatMenu.connectors") return "Connectors";
      if (key === "chatMenu.plugins") return "Plugins";
      return key;
    },
  }),
}));

vi.mock("../chat-plus-menu.css", () => ({}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  connectorState.configured = [];
});

describe("ChatPlusMenu plan mode", () => {
  it("affiche et active le toggle Plan mode", () => {
    const onPlanModeChange = vi.fn();
    const { getByText, getAllByRole, getByRole } = render(
      <ChatPlusMenu
        onFileImport={vi.fn()}
        planModeEnabled={false}
        onPlanModeChange={onPlanModeChange}
      />,
    );

    fireEvent.click(getAllByRole("button")[0]);
    fireEvent.click(getByRole("switch", { name: "Plan mode" }));

    expect(getByText("Prepare a plan")).toBeTruthy();
    expect(onPlanModeChange).toHaveBeenCalledWith(true);
  });

  it("désactive le toggle Plan mode quand il est actif", () => {
    const onPlanModeChange = vi.fn();
    const { getAllByRole, getByRole } = render(
      <ChatPlusMenu
        onFileImport={vi.fn()}
        planModeEnabled
        onPlanModeChange={onPlanModeChange}
      />,
    );

    fireEvent.click(getAllByRole("button")[0]);
    fireEvent.click(getByRole("switch", { name: "Plan mode" }));

    expect(onPlanModeChange).toHaveBeenCalledWith(false);
  });

  it("active un connecteur depuis le switch partagé", () => {
    connectorState.configured = [{
      id: "github",
      display_name: "GitHub",
      status: "connected",
      enabled_in_chat: false,
    }];
    const view = render(
      <ChatPlusMenu
        onFileImport={vi.fn()}
        planModeEnabled={false}
        onPlanModeChange={vi.fn()}
      />,
    );

    fireEvent.click(view.getAllByRole("button")[0]);
    fireEvent.mouseEnter(view.getByText("Connectors").closest("button") as HTMLButtonElement);
    fireEvent.click(view.getByRole("switch", { name: "GitHub" }));

    expect(connectorState.toggleChatEnabled).toHaveBeenCalledWith("github");
  });
});
