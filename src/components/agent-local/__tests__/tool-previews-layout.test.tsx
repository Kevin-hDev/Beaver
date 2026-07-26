import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { ContentPreview } from "../tool-previews";

afterEach(cleanup);

describe("tool preview layout", () => {
  it("garde tout le contenu dans une bulle scrollable", () => {
    const content = Array.from({ length: 24 }, (_, index) => `line ${index + 1}`).join("\n");
    const { container } = render(<ContentPreview content={content} path="notes.md" />);

    const wrapper = container.querySelector(".tp-wrapper");
    const lines = container.querySelectorAll(".tp-line");

    expect(wrapper).not.toBeNull();
    expect(wrapper).toHaveClass("chat-column-surface");
    expect(lines).toHaveLength(24);
    expect(container.querySelector(".tp-prefix")).toBeNull();
  });
});
