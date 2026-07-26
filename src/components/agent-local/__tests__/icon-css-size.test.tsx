import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { ToolStatusIcon } from "../tool-status-icon";
import { FileIcon } from "@/components/file-preview/file-icon";

afterEach(() => cleanup());

describe("CSS icon sizes", () => {
  it("applies CSS variable sizes to tool status images via style", () => {
    const { getByAltText } = render(<ToolStatusIcon size="var(--icon-sm)" />);
    const img = getByAltText("Erreur");

    expect(img).toHaveStyle({ width: "var(--icon-sm)", height: "var(--icon-sm)" });
    expect(img).not.toHaveAttribute("width");
    expect(img).not.toHaveAttribute("height");
  });

  it("applies CSS variable sizes to file icons via style", () => {
    const { container } = render(<FileIcon name="component.tsx" size="var(--icon-sm)" />);
    const img = container.querySelector(".fp-icon");

    expect(img).toHaveStyle({ width: "var(--icon-sm)", height: "var(--icon-sm)" });
    expect(img).not.toHaveAttribute("width");
    expect(img).not.toHaveAttribute("height");
  });
});
