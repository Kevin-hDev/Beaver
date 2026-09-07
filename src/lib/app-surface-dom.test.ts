import { describe, expect, it } from "vitest";
import { isElementInsideInactiveSurface } from "./app-surface-dom";

describe("isElementInsideInactiveSurface", () => {
  it("reconnaît une frontière inactive portée par un ancêtre", () => {
    const root = document.createElement("section");
    root.setAttribute("hidden", "");
    const child = document.createElement("button");
    root.append(child);

    expect(isElementInsideInactiveSurface(child)).toBe(true);
  });

  it("laisse passer un élément sans frontière inactive", () => {
    expect(isElementInsideInactiveSurface(document.createElement("button"))).toBe(false);
  });
});
