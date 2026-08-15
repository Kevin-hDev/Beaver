import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SendStopButton } from "../send-stop-button";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

function drawingOf(state: "send" | "stop" | "confirmStop"): string {
  const { container } = render(<SendStopButton state={state} onSend={vi.fn()} onStop={vi.fn()} />);
  return container.querySelector("svg")?.innerHTML ?? "";
}

describe("SendStopButton", () => {
  it("arrête dès le premier clic pendant la confirmation", () => {
    const onStop = vi.fn();

    render(<SendStopButton state="confirmStop" onSend={vi.fn()} onStop={onStop} />);
    const button = screen.getByRole("button", { name: "agentLocal.stop" });
    expect(button).toHaveAttribute("data-state", "confirmStop");

    fireEvent.click(button);

    expect(onStop).toHaveBeenCalledOnce();
  });

  /* Les trois états partagent leur cadre et ne diffèrent que par leur centre.
     Un dessin qui reviendrait à l'identique rendrait un état invisible. */
  it("donne un dessin distinct à chacun de ses trois états", () => {
    const drawings = [drawingOf("send"), drawingOf("stop"), drawingOf("confirmStop")];

    expect(new Set(drawings).size).toBe(3);
    expect(drawings.every((d) => d.length > 0)).toBe(true);
  });

  it("est inerte quand il n'y a rien à envoyer ni à arrêter", () => {
    render(<SendStopButton state="hidden" onSend={vi.fn()} onStop={vi.fn()} />);

    expect(screen.getByRole("button")).toBeDisabled();
  });
});
