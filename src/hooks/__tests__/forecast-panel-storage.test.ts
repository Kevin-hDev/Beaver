import { beforeEach, describe, expect, it } from "vitest";
import {
  loadForecastPanelValue,
  removeForecastPanelValue,
  saveForecastPanelValue,
} from "../forecast-panel-storage";

describe("forecast panel storage", () => {
  beforeEach(() => localStorage.clear());

  it("conserve au maximum 32 sessions persistées", () => {
    for (let index = 0; index < 33; index += 1) {
      saveForecastPanelValue(`session-${index}`, { index });
    }

    expect(loadForecastPanelValue("session-0")).toBeNull();
    expect(loadForecastPanelValue("session-32")).toEqual({ index: 32 });
  });

  it("refuse les identifiants et états persistés non bornés", () => {
    saveForecastPanelValue("../session", { unsafe: true });
    saveForecastPanelValue("session-too-large", { value: "x".repeat(4_097) });
    localStorage.setItem("fc-panel-session-large", "x".repeat(4_097));

    expect(loadForecastPanelValue("../session")).toBeNull();
    expect(loadForecastPanelValue("session-too-large")).toBeNull();
    expect(loadForecastPanelValue("session-large")).toBeNull();
  });

  it("supprime la valeur et son entrée d'ordre", () => {
    saveForecastPanelValue("session-a", { value: true });
    removeForecastPanelValue("session-a");

    expect(loadForecastPanelValue("session-a")).toBeNull();
    expect(localStorage.getItem("fc-panel-session-order")).toBe("[]");
  });
});
