/// <reference types="node" />

import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const tokens = readFileSync("src/styles/tokens.css", "utf8");
const shell = readFileSync("src/components/internal-browser/browser-shell.css", "utf8");

describe("browser toolbar density", () => {
  it("garde des barres compactes sans réduire leur largeur", () => {
    expect(tokens).toContain("--browser-toolbar-height: 2rem;");
    expect(tokens).toContain("--browser-control-size: 1.75rem;");
    expect(tokens).toContain("--browser-toolbar-padding-y: 0.125rem;");
    expect(tokens).toContain("--browser-field-padding-y: var(--space-xs);");

    expect(shell).toContain("padding: var(--browser-toolbar-padding-y) var(--space-sm);");
    expect(shell).toContain(
      "padding: var(--browser-field-padding-y) var(--space-xs) " +
      "var(--browser-field-padding-y) var(--space-sm);",
    );
    expect(shell).toContain("padding: var(--browser-field-padding-y) var(--space-sm);");
  });
});
