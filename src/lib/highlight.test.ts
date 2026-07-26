import { describe, expect, it } from "vitest";
import { highlightCodeNodes } from "./highlight";

function flatten(nodes: ReturnType<typeof highlightCodeNodes>): string {
  return nodes.map((node) => {
    if (typeof node === "string") return node;
    return flatten(node.children);
  }).join("");
}

describe("highlightCodeNodes", () => {
  it("préserve le code comme texte, sans HTML injecté", () => {
    const code = `<img src=x onerror=alert(1)>`;

    expect(flatten(highlightCodeNodes(code, "typescript"))).toBe(code);
  });
});
