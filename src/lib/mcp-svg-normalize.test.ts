import { describe, expect, it } from "vitest";
import apifyIconSvg from "@/assets/Apify-2/Apify-icon.svg?raw";
import apifyTextSvg from "@/assets/Apify-2/Apify-text.svg?raw";
import canvaSvg from "@/assets/Canva/canva-icon.svg?raw";
import canvaTextSvg from "@/assets/Canva/Canva-text.svg?raw";
import figmaSvg from "@/assets/Figma/Figma.svg?raw";
import githubIconSvg from "@/assets/github/github.svg?raw";
import githubTextSvg from "@/assets/github/github-text.svg?raw";
import huggingfaceSvg from "@/assets/hugging-face/huggingface.svg?raw";
import imessageSvg from "@/assets/IMessage/IMessage_logo.svg?raw";
import linearIconSvg from "@/assets/Linear/Linear-icon.svg?raw";
import linearTextSvg from "@/assets/Linear/Linear-text.svg?raw";
import notionIconSvg from "@/assets/Notion/Notion-icon.svg?raw";
import notionTextSvg from "@/assets/Notion/Notion-text.svg?raw";
import producthuntSvg from "@/assets/Product_Hunt/Product-hunt.svg?raw";
import redditSvg from "@/assets/Reddit/Reddit-icon.svg?raw";
import redditTextSvg from "@/assets/Reddit/Reddit-text.svg?raw";
import sentryIconSvg from "@/assets/Sentry/Sentry-icon.svg?raw";
import sentryTextSvg from "@/assets/Sentry/Sentry-text.svg?raw";
import slackSvg from "@/assets/Slack-2/Slack-icon.svg?raw";
import slackTextSvg from "@/assets/Slack-2/Slack-text.svg?raw";
import vercelIconSvg from "@/assets/Vercel/Vercel-icon.svg?raw";
import vercelTextSvg from "@/assets/Vercel/Vercel-text.svg?raw";
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
    const assets = [
      apifyIconSvg, apifyTextSvg, canvaSvg, canvaTextSvg, figmaSvg,
      githubIconSvg, githubTextSvg, huggingfaceSvg, imessageSvg,
      linearIconSvg, linearTextSvg, notionIconSvg, notionTextSvg,
      producthuntSvg, redditSvg, redditTextSvg, sentryIconSvg,
      sentryTextSvg, slackSvg, slackTextSvg, vercelIconSvg, vercelTextSvg,
    ];

    for (const [index, asset] of assets.entries()) {
      const prepared = prepareMcpSvg(asset, `asset-${index}-`);
      expect(prepared).toContain("<svg");
      expect(prepared).not.toContain("<svg:");
      expect(prepared).not.toContain("<style");
      expect(prepared).not.toContain("style=");
      expect(missingUrlRefs(prepared)).toEqual([]);
    }
  });

  it("removes active content and event handlers before rendering", () => {
    const svg = `
      <svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)">
        <script>alert(1)</script>
        <foreignObject><div>unsafe</div></foreignObject>
        <path id="safe" d="M0 0h10v10z" onclick="alert(1)"/>
      </svg>
    `;

    const prepared = prepareMcpSvg(svg, "safe-");

    expect(prepared).not.toContain("<script");
    expect(prepared).not.toContain("foreignObject");
    expect(prepared).not.toContain("onload");
    expect(prepared).not.toContain("onclick");
    expect(prepared).toContain('id="safe-safe"');
  });

  it("removes external references while keeping local gradient references", () => {
    const svg = `
      <svg xmlns="http://www.w3.org/2000/svg">
        <linearGradient id="local"/>
        <use href="https://example.com/image.svg#shape"/>
        <rect fill="url(https://example.com/image.svg#paint)"/>
        <rect fill="url(#local)"/>
      </svg>
    `;

    const prepared = prepareMcpSvg(svg, "safe-");

    expect(prepared).not.toContain("https://example.com");
    expect(prepared).toContain('fill="url(#safe-local)"');
  });

  it("rejects malformed, document-bearing, oversized, or invalidly scoped SVG", () => {
    expect(prepareMcpSvg("<svg><sty<style>le></svg>", "safe-")).toBe("");
    expect(prepareMcpSvg('<!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg"/>', "safe-")).toBe("");
    expect(prepareMcpSvg(`<svg xmlns="http://www.w3.org/2000/svg">${"x".repeat(256 * 1024)}</svg>`, "safe-")).toBe("");
    expect(prepareMcpSvg('<svg xmlns="http://www.w3.org/2000/svg"/>', 'bad" scope')).toBe("");
  });
});

function missingUrlRefs(svg: string): string[] {
  const ids = new Set([...svg.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]));
  return [...svg.matchAll(/url\(#([^)]+)\)/g)]
    .map((match) => match[1])
    .filter((id) => !ids.has(id));
}
