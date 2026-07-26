import { afterEach, describe, expect, it } from "vitest";
import { applyStoredSettings, parseStoredFontSize } from "../use-settings";

describe("parseStoredFontSize", () => {
  it.each([
    ["100", 18],
    ["112", 20],
    ["125", 22],
    ["137", 24],
    ["150", 24],
  ])("migre l'ancienne valeur %s%% vers %ipx", (raw, expected) => {
    expect(parseStoredFontSize(raw)).toBe(expected);
  });

  it.each([
    ["9", 10],
    ["29", 24],
    ["abc", 18],
    [null, 18],
  ])("normalise la valeur stockée %s", (raw, expected) => {
    expect(parseStoredFontSize(raw)).toBe(expected);
  });
});

describe("applyStoredSettings", () => {
  afterEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-code-theme");
    document.documentElement.style.fontSize = "";
    document.documentElement.style.removeProperty("--font-sans");
  });

  it("applique les réglages visuels stockés avant le rendu des settings", () => {
    localStorage.setItem("clgo-font-size", "125");
    localStorage.setItem("clgo-font-family", "menlo");
    localStorage.setItem("clgo-code-theme", "tokyo-night");

    applyStoredSettings();

    expect(document.documentElement.style.fontSize).toBe("22px");
    expect(document.documentElement.style.getPropertyValue("--font-sans")).toBe(
      'Menlo, "SF Mono", Consolas, monospace',
    );
    expect(document.documentElement.getAttribute("data-code-theme")).toBe("tokyo-night");
  });

  it("retombe sur les valeurs par défaut si le stockage contient des valeurs invalides", () => {
    localStorage.setItem("clgo-font-size", "abc");
    localStorage.setItem("clgo-font-family", "unknown");
    localStorage.setItem("clgo-code-theme", "unknown");

    applyStoredSettings();

    expect(document.documentElement.style.fontSize).toBe("18px");
    expect(document.documentElement.style.getPropertyValue("--font-sans")).toBe(
      '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    );
    expect(document.documentElement.getAttribute("data-code-theme")).toBe("default");
  });
});
