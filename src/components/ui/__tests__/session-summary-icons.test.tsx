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
        <CommitIcon size="var(--icon-md)" />
        <ModificationIcon size="var(--icon-md)" />
        <PlanIcon size="var(--icon-md)" />
        <TodoListIcon size="var(--icon-md)" />
        <SubagentSummaryIcon size="var(--icon-md)" />
      </>,
    );

    const icons = Array.from(container.querySelectorAll("svg"));

    expect(icons).toHaveLength(5);
    expect(icons.map((icon) => icon.getAttribute("viewBox"))).toEqual([
      "0 0 24 24",
      "0 0 24 24",
      "0 0 48 48",
      "0 0 24 24",
      "0 0 32 32",
    ]);
    expect(icons.every((icon) => icon.style.width === "var(--icon-md)")).toBe(true);
    expect(container.querySelector('[fill="#d2d2d2"]')).toBeNull();
  });
});
