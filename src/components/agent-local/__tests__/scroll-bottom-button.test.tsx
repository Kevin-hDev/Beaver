import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { ScrollBottomButton } from "../scroll-bottom-button";

afterEach(cleanup);

describe("ScrollBottomButton", () => {
  it("se pose au-dessus du champ de saisie plutôt que dans le flux", () => {
    const { container } = render(<ScrollBottomButton onClick={vi.fn()} />);

    expect(container.querySelector(".scroll-bottom-btn")).not.toBeNull();
  });

  it("appelle le retour en bas au clic", () => {
    const onClick = vi.fn();
    const { container } = render(<ScrollBottomButton onClick={onClick} />);

    fireEvent.click(container.querySelector("button")!);

    expect(onClick).toHaveBeenCalledTimes(1);
  });
});
