import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { ChatMarkdown } from "../chat-markdown";

const mocks = vi.hoisted(() => ({ open: vi.fn(() => Promise.resolve()) }));

vi.mock("@tauri-apps/plugin-shell", () => ({ open: mocks.open }));

beforeEach(() => {
  mocks.open.mockClear();
  localStorage.setItem("clgo-link-preview", "false");
});

describe("ChatMarkdown", () => {
  it("rend les principaux blocs CommonMark et GFM", () => {
    const { container } = render(
      <div className="chat-md">
        <ChatMarkdown
          content={[
            "# Titre",
            "",
            "## Sous-titre",
            "",
            "**Gras** et *italique* puis ~~barré~~.",
            "",
            "- Premier",
            "- Second",
            "",
            "1. Étape",
            "2. Suite",
            "",
            "---",
            "",
            "> Citation",
            "",
            "| Nom | État |",
            "| --- | --- |",
            "| Test | OK |",
            "",
            "- [x] Terminé",
          ].join("\n")}
        />
      </div>,
    );

    expect(screen.getByRole("heading", { level: 1, name: "Titre" })).toBeTruthy();
    expect(screen.getByRole("heading", { level: 2, name: "Sous-titre" })).toBeTruthy();
    expect(container.querySelector("strong")?.textContent).toBe("Gras");
    expect(container.querySelector("em")?.textContent).toBe("italique");
    expect(container.querySelector("del")?.textContent).toBe("barré");
    expect(container.querySelectorAll("ul li")).toHaveLength(3);
    expect(container.querySelectorAll("ol li")).toHaveLength(2);
    expect(container.querySelector("hr")).toBeTruthy();
    expect(container.querySelector("blockquote")?.textContent).toContain("Citation");
    expect(container.querySelector("table")).toBeTruthy();
    expect(container.querySelector(".task-list-item")).toBeTruthy();
  });

  it("préserve les chips de skills dans les titres et le texte enrichi", () => {
    const { container } = render(
      <div className="chat-md">
        <ChatMarkdown
          content={"## /context7-docs\n\n- **/compress**"}
          skillNames={["context7-docs"]}
          builtInNames={["compress"]}
        />
      </div>,
    );

    expect(container.querySelectorAll(".skill-chip")).toHaveLength(2);
    expect(container.querySelector(".skill-chip-built-in")?.textContent).toBe("compress");
  });

  it("retire le HTML dangereux avant le rendu", () => {
    const { container } = render(
      <ChatMarkdown content={'Texte sûr<script>alert("x")</script><iframe src="https://example.com" />'} />,
    );

    expect(container.querySelector("script")).toBeNull();
    expect(container.querySelector("iframe")).toBeNull();
    expect(container.textContent).toContain("Texte sûr");
  });

  it("ouvre seulement les liens externes autorisés", () => {
    render(
      <ChatMarkdown content={"[Documentation](https://example.com/docs) [Local](/private)"} />,
    );

    fireEvent.click(screen.getByRole("link", { name: "Documentation" }));
    expect(mocks.open).toHaveBeenCalledWith("https://example.com/docs");

    fireEvent.click(screen.getByRole("link", { name: "Local" }));
    expect(mocks.open).toHaveBeenCalledOnce();
  });
});
