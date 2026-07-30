import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const onboardingCss = readFileSync("src/components/onboarding/onboarding.css", "utf8");
const agentImportCss = readFileSync(
  "src/components/agent-import/agent-import.css",
  "utf8",
);
const tokensCss = readFileSync("src/styles/tokens.css", "utf8");

function rule(css: string, selector: string): string {
  const marker = `${selector} {`;
  let offset = 0;
  let body = "";
  while (offset < css.length) {
    const start = css.indexOf(marker, offset);
    if (start === -1) break;
    const bodyStart = start + marker.length;
    const end = css.indexOf("}", bodyStart);
    if (end === -1) break;
    body = css.slice(bodyStart, end);
    offset = end + 1;
  }
  return body;
}

describe("mise en page du splash", () => {
  it("garde les actions fixes et fait défiler uniquement la grille des providers", () => {
    expect(rule(onboardingCss, ".ob-page-api")).toContain("overflow: hidden");
    expect(rule(onboardingCss, ".ob-page-api > *")).toContain("flex: none");
    expect(rule(onboardingCss, ".ob-page-api .ob-provider-grid")).toMatch(
      /flex:\s*0 1 auto/,
    );
    expect(rule(onboardingCss, ".ob-page-api .ob-provider-grid")).toContain(
      "overflow-y: auto",
    );
    expect(rule(onboardingCss, ".ob-actions")).toContain("flex-wrap: wrap");
  });

  it("dimensionne l'import depuis son parent et garde ses listes défilables", () => {
    const shell = rule(agentImportCss, ".aim-onboarding-shell");
    expect(shell).toContain("100%");
    expect(shell).not.toMatch(/\d+vh/);
    expect(rule(agentImportCss, ".aim-wizard > *,\n.aim-detail > *")).toContain(
      "flex: none",
    );
    expect(rule(agentImportCss, ".aim-grid-scroll")).toMatch(/flex:\s*0 1 auto/);
    expect(rule(agentImportCss, ".aim-detail-scroll")).toMatch(/flex:\s*1 1 auto/);
  });

  it("conserve des espacements adaptatifs bornés", () => {
    expect(tokensCss).toMatch(/--onboarding-pad-y:\s*clamp\(/);
    expect(tokensCss).toMatch(/--onboarding-pad-x:\s*clamp\(/);
    expect(tokensCss).toMatch(/--onboarding-stack-gap:\s*clamp\(/);
  });
});
