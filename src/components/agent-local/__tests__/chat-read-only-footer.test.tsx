import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ChatReadOnlyFooter } from "../chat-read-only-footer";

describe("ChatReadOnlyFooter", () => {
  it("conserve son espace lorsque le bouton de défilement apparaît", () => {
    const props = {
      onScrollBottom: vi.fn(),
      showError: false,
      showScrollButton: false,
    };
    const { container, rerender } = render(<ChatReadOnlyFooter {...props} />);
    const footer = container.querySelector(".chat-input-area");

    expect(footer).toBeInTheDocument();
    expect(container.querySelector(".scroll-bottom-btn")).not.toBeInTheDocument();

    rerender(<ChatReadOnlyFooter {...props} showScrollButton />);

    expect(container.querySelector(".chat-input-area")).toBe(footer);
    expect(container.querySelector(".scroll-bottom-btn")).toBeInTheDocument();
  });
});
