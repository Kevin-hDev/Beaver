import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const darkCss = readFileSync("src/styles/themes/dark.css", "utf8");
const lightCss = readFileSync("src/styles/themes/light.css", "utf8");
const emeraldCss = readFileSync("src/styles/themes/emerald-night.css", "utf8");
const cobaltCss = readFileSync("src/styles/themes/cobalt-frost.css", "utf8");
const astralCss = readFileSync("src/styles/themes/astral-mist.css", "utf8");
const crimsonCss = readFileSync("src/styles/themes/crimson-eclipse.css", "utf8");
const toolPreviewsCss = readFileSync("src/components/agent-local/tool-previews.css", "utf8");
const gitDiffCss = readFileSync("src/components/file-preview/git-diff-preview.css", "utf8");
const chatCss = readFileSync("src/components/agent-local/chat.css", "utf8");

function tokenNames(css: string): string[] {
  return [...css.matchAll(/(--[a-z0-9-]+)\s*:/g)]
    .map((match) => match[1])
    .sort();
}

function rgb(hex: string): [number, number, number] {
  const normalized = hex.replace("#", "");
  return [0, 2, 4].map((offset) => Number.parseInt(normalized.slice(offset, offset + 2), 16)) as [number, number, number];
}

function luminance(hex: string): number {
  const channels = rgb(hex).map((channel) => {
    const value = channel / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
}

function contrast(first: string, second: string): number {
  const light = Math.max(luminance(first), luminance(second));
  const dark = Math.min(luminance(first), luminance(second));
  return (light + 0.05) / (dark + 0.05);
}

describe("Emerald Night palette", () => {
  it("définit tous les tokens du thème sombre", () => {
    expect(tokenNames(emeraldCss)).toEqual(tokenNames(darkCss));
  });

  it("maintient des contrastes lisibles pour les textes et actions", () => {
    expect(contrast("#f1e2c3", "#031f1c")).toBeGreaterThanOrEqual(7);
    expect(contrast("#bdc7b4", "#031f1c")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#4ee07a", "#062318")).toBeGreaterThanOrEqual(4.5);
  });

  it("conserve un orange distinct et lisible pour les anciennes lignes", () => {
    expect(emeraldCss).toContain("--diff-old: #f97316;");
    expect(contrast("#f97316", "#031f1c")).toBeGreaterThanOrEqual(4.5);
  });
});

describe("Cobalt Frost palette", () => {
  it("définit tous les tokens du thème clair", () => {
    expect(tokenNames(cobaltCss)).toEqual(tokenNames(lightCss));
  });

  it("maintient des contrastes lisibles pour les textes et actions", () => {
    expect(contrast("#101828", "#f7f9fd")).toBeGreaterThanOrEqual(7);
    expect(contrast("#344054", "#f7f9fd")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#ffffff", "#075dcc")).toBeGreaterThanOrEqual(4.5);
  });

  it("distingue clairement les nouvelles et anciennes lignes", () => {
    expect(cobaltCss).toContain("--diff-new: #075dcc;");
    expect(cobaltCss).toContain("--diff-old: #c5303c;");
    expect(contrast("#075dcc", "#f7f9fd")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#c5303c", "#f7f9fd")).toBeGreaterThanOrEqual(4.5);
  });

  it("applique les tokens de diff à toutes les parties des previews", () => {
    expect(toolPreviewsCss).toContain("background: var(--diff-add-bg);");
    expect(toolPreviewsCss).toContain("background: var(--diff-del-bg);");
    expect(toolPreviewsCss).toContain("color: var(--diff-new);");
    expect(gitDiffCss).toContain("color: var(--diff-new);");
  });
});

describe("Astral Mist palette", () => {
  it("définit tous les tokens du thème sombre", () => {
    expect(tokenNames(astralCss)).toEqual(tokenNames(darkCss));
  });

  it("maintient des contrastes lisibles pour les textes et actions", () => {
    expect(contrast("#dce4ee", "#030817")).toBeGreaterThanOrEqual(7);
    expect(contrast("#aab9cb", "#030817")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#9ab5d5", "#07111f")).toBeGreaterThanOrEqual(4.5);
  });

  it("distingue les nouvelles lignes gris-bleu des anciennes lignes rouges", () => {
    expect(astralCss).toContain("--diff-new: #9cb6d3;");
    expect(astralCss).toContain("--diff-old: #f07680;");
    expect(contrast("#9cb6d3", "#030817")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#f07680", "#030817")).toBeGreaterThanOrEqual(4.5);
  });
});

describe("Crimson Eclipse palette", () => {
  it("définit tous les tokens du thème sombre", () => {
    expect(tokenNames(crimsonCss)).toEqual(tokenNames(darkCss));
  });

  it("maintient des contrastes lisibles pour les textes et actions", () => {
    expect(contrast("#e4e4e7", "#070709")).toBeGreaterThanOrEqual(7);
    expect(contrast("#b4b4bb", "#070709")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#ff3038", "#130608")).toBeGreaterThanOrEqual(4.5);
  });

  it("conserve le vert pour les nouvelles lignes et le rouge pour les anciennes", () => {
    expect(crimsonCss).toContain("--diff-new: #35c878;");
    expect(crimsonCss).toContain("--diff-old: #ff5a63;");
    expect(contrast("#35c878", "#070709")).toBeGreaterThanOrEqual(4.5);
    expect(contrast("#ff5a63", "#070709")).toBeGreaterThanOrEqual(4.5);
  });
});

describe("Chat composer relief", () => {
  it("sépare le champ du fond avec deux contours opposés", () => {
    expect(chatCss).toContain("-1px -1px 1px var(--chat-composer-edge-highlight)");
    expect(chatCss).toContain("1px 1px 1px var(--chat-composer-edge-shade)");
  });
});
