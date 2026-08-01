import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ToolStatusIcon } from "../tool-status-icon";

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("ToolStatusIcon", () => {
  it("affiche puis copie le message d'erreur fourni", async () => {
    vi.useFakeTimers();
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    const { getByAltText } = render(
      <ToolStatusIcon message="L’état actuel empêche cette opération." />,
    );

    fireEvent.mouseEnter(getByAltText("Erreur").parentElement!);
    await act(() => vi.advanceTimersByTime(700));

    expect(screen.getByText("L’état actuel empêche cette opération.")).toBeTruthy();
    await act(async () => {
      fireEvent.click(screen.getByRole("button"));
      await Promise.resolve();
    });
    expect(writeText).toHaveBeenCalledWith("L’état actuel empêche cette opération.");
  });
});
