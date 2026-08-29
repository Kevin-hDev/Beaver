import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ProviderIcon } from "./provider-icons";

describe("ProviderIcon", () => {
  it("uses the dedicated Anthropic asset and accessible name", () => {
    render(<ProviderIcon providerId="anthropic" displayName="Anthropic Claude" />);

    const image = screen.getByRole("img", { name: "Anthropic Claude" });
    expect(image.getAttribute("src")).toMatch(/^data:image\/svg\+xml/);
  });
});
