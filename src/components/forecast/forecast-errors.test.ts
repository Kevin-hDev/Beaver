import { describe, expect, it } from "vitest";
import { forecastLaunchErrorKey } from "./forecast-errors";

describe("forecastLaunchErrorKey", () => {
  it("explique comment libérer de la place quand le stockage est plein", () => {
    expect(forecastLaunchErrorKey("forecast-capacity-reached"))
      .toBe("forecast.errors.capacityReached");
  });

  it("ne révèle pas les autres erreurs internes", () => {
    expect(forecastLaunchErrorKey("internal path detail"))
      .toBe("forecast.errors.launchFailed");
  });
});
