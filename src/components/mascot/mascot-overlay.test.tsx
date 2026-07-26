import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { MascotOverlay } from "./mascot-overlay";

const windowMocks = vi.hoisted(() => ({
  onMoved: vi.fn().mockResolvedValue(() => {}),
  setCursorIcon: vi.fn().mockResolvedValue(undefined),
  setPosition: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => windowMocks,
}));

describe("interaction avec la mascotte", () => {
  beforeEach(() => {
    windowMocks.onMoved.mockClear();
    windowMocks.setCursorIcon.mockClear();
    windowMocks.setPosition.mockClear();
  });

  it("déplace la fenêtre et joue les poses directionnelles pendant le glisser", async () => {
    const { container } = render(<MascotOverlay />);
    const mascot = screen.getByRole("img");
    const sprite = container.querySelector("[data-animation]");

    await waitFor(() => expect(windowMocks.setCursorIcon).toHaveBeenCalledWith("grab"));
    fireEvent.pointerDown(mascot, {
      button: 0,
      clientX: 20,
      clientY: 30,
      pointerId: 7,
      screenX: 220,
      screenY: 330,
    });
    expect(sprite).toHaveAttribute("data-animation", "grabbed");

    fireEvent.pointerMove(mascot, {
      pointerId: 7,
      screenX: 250,
      screenY: 330,
    });

    await waitFor(() => expect(windowMocks.setPosition).toHaveBeenCalledOnce());
    expect(windowMocks.setCursorIcon).toHaveBeenCalledWith("grabbing");
    expect(sprite).toHaveAttribute("data-animation", "move-right");

    fireEvent.pointerUp(mascot, {
      pointerId: 7,
      screenX: 250,
      screenY: 330,
    });

    expect(windowMocks.setCursorIcon).toHaveBeenLastCalledWith("grab");
    expect(sprite).toHaveAttribute("data-animation", "dropped");
  });

  it("ignore les autres boutons de la souris", () => {
    render(<MascotOverlay />);
    fireEvent.pointerDown(screen.getByRole("img"), { button: 2 });

    expect(windowMocks.setPosition).not.toHaveBeenCalled();
  });
});
