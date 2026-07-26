import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { PanelToggleIcon, SessionSummaryIcon, TerminalIcon } from "../chat-header-icons";

describe("chat header custom icons", () => {
  it("renders the three supplied drawings with theme-aware colors", () => {
    const { container } = render(
      <>
        <SessionSummaryIcon size="var(--chrome-icon-md)" />
        <PanelToggleIcon size="var(--chrome-icon-md)" />
        <TerminalIcon size="var(--chrome-icon-lg)" />
      </>,
    );

    const icons = Array.from(container.querySelectorAll("svg"));

    expect(icons).toHaveLength(3);
    expect(icons.map((icon) => icon.getAttribute("viewBox"))).toEqual([
      "0 0 16 16",
      "0 0 24 24",
      "0 0 24 24",
    ]);
    expect(icons[0]).toHaveStyle({ width: "var(--chrome-icon-md)" });
    expect(icons[0].querySelector('[fill="currentColor"]')).not.toBeNull();
    expect(icons[0].querySelector('[fill="currentColor"]')?.getAttribute("d")).toContain("M5.5 5");
    expect(icons[1]).toHaveAttribute("stroke", "currentColor");
    expect(icons[2]).toHaveAttribute("stroke", "currentColor");
  });
});
