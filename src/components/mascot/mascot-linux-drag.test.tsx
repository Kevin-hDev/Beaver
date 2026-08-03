/* @vitest-environment jsdom */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { MascotOverlay } from "./mascot-overlay";

const windowMocks = vi.hoisted(() => ({
  onMoved: vi.fn().mockResolvedValue(() => {}),
  setCursorIcon: vi.fn().mockResolvedValue(undefined),
  setPosition: vi.fn().mockResolvedValue(undefined),
  startDragging: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@/lib/platform", () => ({ IS_LINUX: true }));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => windowMocks,
}));

describe("déplacement Linux de la mascotte", () => {
  beforeEach(() => {
    windowMocks.setPosition.mockClear();
    windowMocks.startDragging.mockClear();
  });

  it("confie le déplacement au système sans imposer de coordonnées globales", async () => {
    render(<MascotOverlay />);
    const mascot = screen.getByRole("img");

    fireEvent.pointerDown(mascot, {
      button: 0,
      clientX: 20,
      clientY: 30,
      pointerId: 7,
      screenX: 220,
      screenY: 330,
    });

    await waitFor(() => expect(windowMocks.startDragging).toHaveBeenCalledOnce());
    fireEvent.pointerMove(mascot, {
      pointerId: 7,
      screenX: 250,
      screenY: 330,
    });
    expect(windowMocks.setPosition).not.toHaveBeenCalled();
  });
});
