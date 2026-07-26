import { describe, expect, it } from "vitest";
import { buildForecastFilterChips } from "./forecast-filter-chip";
import type { ForecastLayerGroup } from "./forecast-layer-matrix";

function group(id: string, itemIds: string[]): ForecastLayerGroup {
  return {
    id,
    titleKey: `t.${id}`,
    items: itemIds.map((itemId) => ({
      id: itemId,
      label: itemId,
      interactive: true,
    })),
  };
}

describe("buildForecastFilterChips", () => {
  it("maps series and uncertainty items to their fixed tokens", () => {
    const chips = buildForecastFilterChips([
      group("series", ["history", "forecast"]),
      group("uncertainty", ["confidence"]),
    ]);

    expect(chips.get("history")).toEqual({
      color: "var(--fc-line-history)",
      shape: "dot",
    });
    expect(chips.get("forecast")).toEqual({
      color: "var(--fc-line-predict)",
      shape: "dot",
    });
    expect(chips.get("confidence")).toEqual({
      color: "var(--fc-band-90)",
      shape: "band",
    });
  });

  it("rotates scenario tokens and continues into the comparisons group", () => {
    const chips = buildForecastFilterChips([
      group("scenarios", ["scenario-a", "scenario-b", "scenario-c", "scenario-d"]),
      group("comparisons", ["scenario-ensemble"]),
    ]);

    expect(chips.get("scenario-a")?.color).toBe("var(--fc-scenario-a)");
    expect(chips.get("scenario-b")?.color).toBe("var(--fc-scenario-b)");
    expect(chips.get("scenario-c")?.color).toBe("var(--fc-scenario-c)");
    expect(chips.get("scenario-d")?.color).toBe("var(--fc-scenario-a)");
    expect(chips.get("scenario-ensemble")?.color).toBe("var(--fc-scenario-b)");
  });

  it("rotates variable tokens independently", () => {
    const chips = buildForecastFilterChips([
      group("scenarios", ["scenario-a"]),
      group("variables", ["variable-x", "variable-y", "variable-z", "variable-w", "variable-v"]),
    ]);

    expect(chips.get("variable-x")?.color).toBe("var(--fc-variable-a)");
    expect(chips.get("variable-w")?.color).toBe("var(--fc-variable-d)");
    expect(chips.get("variable-v")?.color).toBe("var(--fc-variable-a)");
  });

  it("leaves unknown items without a chip", () => {
    const chips = buildForecastFilterChips([group("events", ["mystery"])]);
    expect(chips.has("mystery")).toBe(false);
  });
});
