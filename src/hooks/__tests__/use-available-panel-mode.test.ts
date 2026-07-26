import { describe, expect, it } from "vitest";
import { resolveAvailablePanelMode } from "../use-available-panel-mode";

describe("resolveAvailablePanelMode", () => {
  it("revient à l'aperçu quand le coffre bloque le navigateur", () => {
    expect(resolveAvailablePanelMode("browser", { status: "unavailable" })).toBe("preview");
    expect(resolveAvailablePanelMode("browser", { status: "hidden" })).toBe("preview");
  });

  it("restaure le navigateur dès que CEF est prêt", () => {
    expect(resolveAvailablePanelMode("browser", {
      status: "ready",
      engineVersion: "150.0.0+150.0.10",
    })).toBe("browser");
  });
});
