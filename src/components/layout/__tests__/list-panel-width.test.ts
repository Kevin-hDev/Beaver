import { describe, expect, it } from "vitest";
import { cssLengthToPx, nextListPanelWidth } from "../list-panel-width";

describe("list panel width", () => {
  it("convertit les longueurs CSS utiles en pixels", () => {
    expect(cssLengthToPx("189px", 18)).toBe(189);
    expect(cssLengthToPx("10.5rem", 18)).toBe(189);
    expect(cssLengthToPx("240", 18)).toBe(240);
    expect(cssLengthToPx("auto", 18)).toBeNull();
  });

  it("permet d'agrandir puis de reduire jusqu'a la largeur de base", () => {
    expect(nextListPanelWidth(189, 189, 40)).toBe(229);
    expect(nextListPanelWidth(240, 189, -20)).toBe(220);
    expect(nextListPanelWidth(240, 189, -80)).toBeNull();
  });
});
