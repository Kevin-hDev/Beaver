import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { ProviderIcon } from "./provider-icons";

describe("ProviderIcon", () => {
  it("uses the dedicated Anthropic asset and accessible name", () => {
    render(<ProviderIcon providerId="anthropic" displayName="Anthropic Claude" />);

    const image = screen.getByRole("img", { name: "Anthropic Claude" });
    expect(image.getAttribute("src")).toMatch(/^data:image\/svg\+xml/);
  });

  it("uses the byte-identical Qwen provider asset", () => {
    // eslint-disable-next-line security/detect-non-literal-fs-filename -- fixed repository asset
    const asset = readFileSync(resolve(process.cwd(), "src/assets/providers/qwen.svg"));
    expect(createHash("sha256").update(asset).digest("hex"))
      .toBe("8d45316eaac23168daaead7904b32b78dd891fdb60716121b0207c324fa4ecff");

    render(<ProviderIcon providerId="qwen" displayName="Qwen" />);
    expect(screen.getByRole("img", { name: "Qwen" })).toBeTruthy();
  });
});
