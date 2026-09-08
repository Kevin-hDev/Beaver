/// <reference types="node" />

import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const tokens = readFileSync("src/styles/tokens.css", "utf8");
const shell = readFileSync("src/components/internal-browser/browser-shell.css", "utf8");

describe("browser toolbar density", () => {
  it("relie chaque hauteur navigateur au palier compact unique", () => {
    // Les deux barres utilisent le palier compact partagé de l’application ;
    // un bouton de 28 px dans le champ imposerait une hauteur supérieure à celle demandée.
    expect(tokens).toMatch(/--compact-control-height: 24px;/);
    expect(tokens).toMatch(/--browser-control-size: var\(--compact-control-height\);/);
    expect(tokens).toMatch(/--browser-field-height: var\(--compact-control-height\);/);
    expect(tokens).toMatch(/--browser-tab-height: var\(--compact-control-height\);/);
    expect(tokens).toMatch(/--browser-toolbar-padding-y: calc\(var\(--chrome-1\) \/ 2\);/);
    expect(tokens).toMatch(/--browser-toolbar-border-width: 1px;/);
    expect(tokens).toMatch(
      /--browser-toolbar-height: calc\(var\(--browser-control-size\) \+ 2 \* var\(--browser-toolbar-padding-y\) \+ var\(--browser-toolbar-border-width\)\);/,
    );
    expect(tokens).toMatch(/--browser-control-line-height: 1\.1;/);
    expect(tokens).not.toMatch(/--browser-field-padding-y:/);

    for (const token of [
      "--browser-control-size",
      "--browser-field-height",
      "--browser-tab-height",
      "--browser-toolbar-padding-y",
      "--browser-toolbar-border-width",
      "--browser-toolbar-height",
      "--browser-control-line-height",
    ]) {
      expect(
        tokens.split("\n").filter((line) => line.trimStart().startsWith(`${token}:`)),
      ).toHaveLength(1);
    }

    expect(shell).toMatch(/height: var\(--browser-toolbar-height\);/);
    expect(shell).not.toMatch(/min-height: var\(--browser-toolbar-height\);/);
    expect(shell).toMatch(/box-sizing: border-box;/);
    expect(shell).toMatch(/border-bottom: var\(--browser-toolbar-border-width\) solid var\(--edge\);/);
    expect(shell).toMatch(/\.ib-tab\s*{[^}]*height: var\(--browser-tab-height\);/s);
    expect(shell).toMatch(/\.ib-tab-select\s*{[^}]*height: 100%;[^}]*padding: 0 var\(--space-xs\) 0 var\(--space-sm\);/s);
    expect(shell).toMatch(/\.ib-tab-close\s*{[^}]*width: var\(--space-lg\);[^}]*height: var\(--browser-tab-height\);/s);
    expect(shell).toMatch(/\.ib-address-form\s*{[^}]*height: var\(--browser-field-height\);/s);
    expect(shell).toMatch(/\.ib-address-form\s*{[^}]*border: var\(--browser-toolbar-border-width\) solid var\(--edge\);/s);
    expect(shell).toMatch(/\.ib-address-input\s*{[^}]*height: 100%;[^}]*padding: 0 var\(--space-sm\);/s);
    expect(shell).toMatch(/\.ib-address-go\s*{[^}]*height: 100%;[^}]*aspect-ratio: 1;/s);
    expect(shell.match(/line-height: var\(--browser-control-line-height\);/g)).toHaveLength(2);
    // Aucun cadre de focus propre au navigateur : choix explicite de l’interface.
    expect(shell).not.toMatch(/outline:\s*var\(--focus-visible-ring-width\)/);
    expect(shell).not.toContain(".ib-address-form:focus-within");
  });
});
