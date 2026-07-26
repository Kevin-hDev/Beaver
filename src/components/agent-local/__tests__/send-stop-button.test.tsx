import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SendStopButton } from "../send-stop-button";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("SendStopButton", () => {
  it("affiche ESC pendant la confirmation d'arrêt", () => {
    const onStop = vi.fn();

    render(<SendStopButton state="confirmStop" onSend={vi.fn()} onStop={onStop} />);

    expect(screen.getByText("ESC")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "agentLocal.stop" }));

    expect(onStop).toHaveBeenCalledOnce();
  });
});
