import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import {
  CommitIcon,
  ModificationIcon,
  PlanIcon,
  SubagentSummaryIcon,
  TodoListIcon,
} from "../session-summary-icons";

describe("session summary custom icons", () => {
  it("renders the five supplied drawings with theme-aware colors", () => {
    const { container } = render(
      <>
        <CommitIcon />
        <ModificationIcon />
        <PlanIcon />
        <TodoListIcon />
        <SubagentSummaryIcon />
      </>,
    );

    const icons = Array.from(container.querySelectorAll("svg"));

    expect(icons).toHaveLength(5);
    expect(icons.map((icon) => icon.getAttribute("viewBox"))).toEqual([
      "0 0 24 24",
      "0 0 24 24",
      "0 0 24 24",
      "0 0 640 640",
      "0 0 24 24",
    ]);
    /* Les six rangées de la bulle partagent une seule taille : si un dessin
       échappe au token, il grandit tout seul au prochain réglage. */
    expect(icons.every((icon) => icon.style.width === "var(--summary-row-icon-size)")).toBe(true);
    expect(icons.every((icon) => icon.getAttribute("aria-hidden") === "true")).toBe(true);
    expect(container.querySelector("[fill^='#']")).toBeNull();
    expect(container.querySelector("[stroke^='#']")).toBeNull();
  });

  it("keeps an explicit size overridable", () => {
    const { container } = render(<PlanIcon size="var(--icon-lg)" />);
    expect(container.querySelector("svg")?.style.width).toBe("var(--icon-lg)");
  });
});
