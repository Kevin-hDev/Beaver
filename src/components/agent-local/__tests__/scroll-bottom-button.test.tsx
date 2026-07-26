import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/react";
import { ScrollBottomButton } from "../scroll-bottom-button";

afterEach(cleanup);

describe("ScrollBottomButton", () => {
  it("rend la variante inline pour le pied du champ de saisie", () => {
    const { container } = render(<ScrollBottomButton variant="inline" onClick={vi.fn()} />);

    expect(container.querySelector(".scroll-bottom-btn-inline")).not.toBeNull();
  });

  it("appelle le retour en bas au clic", () => {
    const onClick = vi.fn();
    const { container } = render(<ScrollBottomButton onClick={onClick} />);

    fireEvent.click(container.querySelector("button")!);

    expect(onClick).toHaveBeenCalledTimes(1);
  });
});
