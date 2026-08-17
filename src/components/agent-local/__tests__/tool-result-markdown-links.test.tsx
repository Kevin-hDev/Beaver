import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";

const opened: string[] = [];

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: (url: string) => {
    opened.push(url);
    return Promise.resolve();
  },
}));

import { ToolResultMarkdown } from "../tool-result-markdown";

describe("ToolResultMarkdown - liens hostiles", () => {
  beforeEach(() => {
    opened.length = 0;
  });

  it("neutralise un lien javascript: en markdown", () => {
    render(<ToolResultMarkdown content={"[clique](javascript:alert(1))"} />);

    const link = screen.getByRole("link", { name: "clique" });
    fireEvent.click(link);

    expect(opened).toHaveLength(0);
  });

  it("neutralise un lien javascript: en HTML brut", () => {
    render(<ToolResultMarkdown content={'<a href="javascript:alert(1)">clique</a>'} />);

    const link = screen.getByRole("link", { name: "clique" });
    fireEvent.click(link);

    expect(opened).toHaveLength(0);
  });

  it("n'exécute jamais un gestionnaire on* injecté en HTML brut", () => {
    const { container } = render(
      <ToolResultMarkdown content={'<img src="x" onerror="window.__pwned = 1"><a href="https://ok.test">ok</a>'} />,
    );

    expect(container.querySelector("img")?.getAttribute("onerror")).toBeNull();
    expect((window as unknown as Record<string, unknown>).__pwned).toBeUndefined();
  });

  it("ouvre normalement un lien https", () => {
    render(<ToolResultMarkdown content={"[doc](https://example.com)"} />);

    fireEvent.click(screen.getByRole("link", { name: "doc" }));

    expect(opened).toEqual(["https://example.com/"]);
  });
});
