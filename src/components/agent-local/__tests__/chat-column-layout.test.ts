import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const tokensCss = readFileSync("src/styles/tokens.css", "utf8");
const chatCss = readFileSync("src/components/agent-local/chat.css", "utf8");
const columnCss = readFileSync("src/components/agent-local/chat-column.css", "utf8");

describe("chat column layout", () => {
  it("partage une marge latérale compacte de 18px", () => {
    expect(tokensCss).toContain("--chat-column-gutter: 18px;");
    expect(chatCss).toContain("padding: 0 var(--chat-column-gutter)");
  });

  it("dimensionne les surfaces depuis la largeur du chat", () => {
    expect(columnCss).toContain(".chat-zone {");
    expect(columnCss).toContain("container-type: inline-size;");
    expect(columnCss).toContain("100cqi - var(--chat-column-gutter) - var(--chat-column-gutter)");
    expect(columnCss).toContain("max-width: var(--chat-column-width);");
  });
});
