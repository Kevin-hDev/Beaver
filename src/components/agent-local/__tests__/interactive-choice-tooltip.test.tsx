import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { InteractiveChoiceTooltip } from "../interactive-choice-tooltip";

afterEach(cleanup);

describe("InteractiveChoiceTooltip", () => {
  it("affiche le texte complet quand il est tronqué horizontalement", () => {
    const fullText = "A very long option label";
    render(
      <InteractiveChoiceTooltip fullText={fullText}>
        <span>{fullText}</span>
      </InteractiveChoiceTooltip>,
    );

    const text = screen.getByText(fullText);
    Object.defineProperty(text, "scrollWidth", { configurable: true, value: 240 });
    Object.defineProperty(text, "clientWidth", { configurable: true, value: 80 });

    fireEvent.mouseEnter(text.parentElement!);

    expect(screen.getByRole("tooltip").textContent).toBe(fullText);
  });
});
