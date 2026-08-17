import { beforeEach, describe, expect, it, vi } from "vitest";

const opened: string[] = [];

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: (url: string) => {
    opened.push(url);
    return Promise.resolve();
  },
}));

import { openExternalLink } from "../open-external-link";

describe("openExternalLink", () => {
  beforeEach(() => {
    opened.length = 0;
  });

  it("ouvre les liens http et https", () => {
    expect(openExternalLink("https://example.com/page")).toBe(true);
    expect(openExternalLink("http://example.com")).toBe(true);
    expect(opened).toHaveLength(2);
  });

  it("refuse le protocole javascript:", () => {
    expect(openExternalLink("javascript:alert(1)")).toBe(false);
    expect(opened).toHaveLength(0);
  });

  it("refuse les protocoles file: et data:", () => {
    expect(openExternalLink("file:///etc/passwd")).toBe(false);
    expect(openExternalLink("data:text/html,<script>alert(1)</script>")).toBe(false);
    expect(opened).toHaveLength(0);
  });

  it("refuse les variantes avec majuscules et espaces", () => {
    expect(openExternalLink("  javascript:alert(1)")).toBe(false);
    expect(openExternalLink("JaVaScRiPt:alert(1)")).toBe(false);
    expect(opened).toHaveLength(0);
  });

  it("refuse les liens relatifs ou invalides", () => {
    expect(openExternalLink("/etc/passwd")).toBe(false);
    expect(openExternalLink("pas une url")).toBe(false);
    expect(openExternalLink("")).toBe(false);
    expect(opened).toHaveLength(0);
  });

  it("refuse les liens démesurés", () => {
    const long = `https://example.com/${"a".repeat(2048)}`;
    expect(openExternalLink(long)).toBe(false);
    expect(opened).toHaveLength(0);
  });
});
