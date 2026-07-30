import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it } from "vitest";

const onboardingCss = readFileSync("src/components/onboarding/onboarding.css", "utf8");
const agentImportCss = readFileSync(
  "src/components/agent-import/agent-import.css",
  "utf8",
);
const tokensCss = readFileSync("src/styles/tokens.css", "utf8");

function normalizeSelector(selector: string): string {
  return selector.trim().replace(/\s+/g, " ");
}

function visitRules(rules: CSSRuleList, selector: string, matches: CSSStyleRule[]) {
  const expected = normalizeSelector(selector);
  for (const candidate of Array.from(rules)) {
    if (candidate instanceof CSSStyleRule) {
      const selectors = candidate.selectorText.split(",").map(normalizeSelector);
      if (selectors.includes(expected)) matches.push(candidate);
    }
    if ("cssRules" in candidate) {
      visitRules((candidate as CSSGroupingRule).cssRules, selector, matches);
    }
  }
}

function matchingRules(css: string, selector: string): CSSStyleRule[] {
  const element = document.createElement("style");
  element.textContent = css;
  document.head.append(element);
  const matches: CSSStyleRule[] = [];
  visitRules(element.sheet?.cssRules ?? ({} as CSSRuleList), selector, matches);
  expect(matches.length, `règle CSS ${selector}`).toBeGreaterThan(0);
  return matches;
}

function property(css: string, selector: string, name: string): string {
  const values = matchingRules(css, selector)
    .map((rule) => rule.style.getPropertyValue(name).trim())
    .filter(Boolean);
  expect(values, `${selector} doit définir ${name} une seule fois`).toHaveLength(1);
  return values[0];
}

afterEach(() => {
  document.head.querySelectorAll("style").forEach((element) => element.remove());
});

describe("contrat CSS du splash", () => {
  it("réserve la place des actions et limite le défilement à la grille", () => {
    expect(property(onboardingCss, ".ob-page-api", "overflow")).toBe("hidden");
    expect(property(onboardingCss, ".ob-page-api > *", "flex")).toBe("0 0 auto");
    expect(property(onboardingCss, ".ob-page-api .ob-provider-grid", "flex")).toBe(
      "0 1 auto",
    );
    expect(
      property(onboardingCss, ".ob-page-api .ob-provider-grid", "overflow-y"),
    ).toBe("auto");
    expect(property(onboardingCss, ".ob-actions", "flex-wrap")).toBe("wrap");
  });

  it("dimensionne l'import depuis son parent et garde ses listes défilables", () => {
    const height = property(agentImportCss, ".aim-onboarding-shell", "height");
    expect(height).toContain("100%");
    expect(height).not.toContain("vh");
    expect(property(agentImportCss, ".aim-wizard > *", "flex")).toBe("0 0 auto");
    expect(property(agentImportCss, ".aim-detail > *", "flex")).toBe("0 0 auto");
    expect(property(agentImportCss, ".aim-grid-scroll", "flex")).toBe("0 1 auto");
    expect(property(agentImportCss, ".aim-detail-scroll", "flex")).toBe("1 1 auto");
  });

  it("conserve des espacements adaptatifs bornés", () => {
    for (const token of [
      "--onboarding-pad-y",
      "--onboarding-pad-x",
      "--onboarding-stack-gap",
    ]) {
      expect(property(tokensCss, ":root", token)).toMatch(/^clamp\(/);
    }
  });
});
