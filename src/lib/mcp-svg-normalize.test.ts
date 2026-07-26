import { describe, expect, it } from "vitest";
import canvaSvg from "@/assets/Canva/canva-icon.svg?raw";
import imessageSvg from "@/assets/IMessage/IMessage_logo.svg?raw";
import producthuntSvg from "@/assets/Product_Hunt/Product-hunt.svg?raw";
import redditSvg from "@/assets/Reddit/Reddit-icon.svg?raw";
import slackSvg from "@/assets/Slack-2/Slack-icon.svg?raw";
import { prepareMcpSvg } from "./mcp-svg-normalize";

describe("prepareMcpSvg", () => {
  it("inlines class styles as SVG attributes", () => {
    const svg = `
      <svg viewBox="0 0 10 10">
        <style>.st0{fill:#FF6154}.st1{fill:#fff}</style>
        <path class="st0" d="M0 0h10v10z"/>
        <path class="st1" d="M1 1h8v8z"/>
      </svg>
    `;

    const prepared = prepareMcpSvg(svg, "ph-test-");

    expect(prepared).not.toContain("<style");
    expect(prepared).toContain('class="ph-test-st0"');
    expect(prepared).toContain('fill="#FF6154"');
    expect(prepared).toContain('fill="#fff"');
  });

  it("inlines style attributes used by gradients", () => {
    const svg = `
      <svg viewBox="0 0 10 10">
        <linearGradient id="g"><stop offset="0" style="stop-color:#0cbd2a;stop-opacity:1"/></linearGradient>
        <rect style="fill:url(#g);stroke:none" width="10" height="10"/>
      </svg>
    `;

    const prepared = prepareMcpSvg(svg, "im-test-");

    expect(prepared).not.toContain("style=");
    expect(prepared).toContain('id="im-test-g"');
    expect(prepared).toContain('stop-color="#0cbd2a"');
    expect(prepared).toContain('stop-opacity="1"');
    expect(prepared).toContain('fill="url(#im-test-g)"');
  });

  it("does not double-prefix xlink hrefs", () => {
    const svg = `
      <svg xmlns:xlink="http://www.w3.org/1999/xlink">
        <linearGradient id="base"/>
        <linearGradient id="copy" xlink:href="#base"/>
        <use href="#copy"/>
      </svg>
    `;

    const prepared = prepareMcpSvg(svg, "scoped-");

    expect(prepared).toContain('xlink:href="#scoped-base"');
    expect(prepared).not.toContain('xlink:href="#scoped-scoped-base"');
    expect(prepared).toContain('href="#scoped-copy"');
  });

  it("removes inline styling from brand assets that broke in release", () => {
    const assets = [canvaSvg, imessageSvg, producthuntSvg, redditSvg, slackSvg];

    for (const [index, asset] of assets.entries()) {
      const prepared = prepareMcpSvg(asset, `asset-${index}-`);
      expect(prepared).not.toContain("<style");
      expect(prepared).not.toContain("style=");
      expect(missingUrlRefs(prepared)).toEqual([]);
    }
  });
});

function missingUrlRefs(svg: string): string[] {
  const ids = new Set([...svg.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]));
  return [...svg.matchAll(/url\(#([^)]+)\)/g)]
    .map((match) => match[1])
    .filter((id) => !ids.has(id));
}
