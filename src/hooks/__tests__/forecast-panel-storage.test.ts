import { beforeEach, describe, expect, it } from "vitest";
import {
  loadForecastPanelValue,
  saveForecastPanelValue,
  withBoundedPanelState,
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

  it("borne également le cache en mémoire", () => {
    let states: Record<string, number> = {};
    for (let index = 0; index < 33; index += 1) {
      states = withBoundedPanelState(states, `session-${index}`, index);
    }

    expect(Object.keys(states)).toHaveLength(32);
    expect(states["session-0"]).toBeUndefined();
    expect(states["session-32"]).toBe(32);
  });

  it("refuse les identifiants et états persistés non bornés", () => {
    saveForecastPanelValue("../session", { unsafe: true });
    localStorage.setItem("fc-panel-session-large", "x".repeat(4_097));

    expect(loadForecastPanelValue("../session")).toBeNull();
    expect(loadForecastPanelValue("session-large")).toBeNull();
  });
});
