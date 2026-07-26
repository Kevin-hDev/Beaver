import { describe, expect, it } from "vitest";

import { BRAND } from "./brand";

describe("public brand", () => {
  it("exposes the Beaver public identity", () => {
    expect(BRAND).toEqual({
      displayName: "Beaver",
      repository: "Kevin-hDev/Beaver",
      userAgentProduct: "Beaver",
    });
  });

  it("does not leak legacy or abandoned names", () => {
    const publicValues = Object.values(BRAND).join(" ").toLowerCase();

    expect(publicValues).not.toContain("beavry");
    expect(publicValues).not.toContain(["cl", "go", "dash"].join("-"));
    expect(publicValues).not.toContain(["cl", "go"].join("-"));
  });
});
