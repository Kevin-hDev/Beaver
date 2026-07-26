import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ForecastViewFilters } from "./forecast-view-filters";
import { ForecastScenarioMenuSelect } from "./sections/forecast-scenario-menu-select";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("Forecast floating menus", () => {
  it("uses the shared compact button and escapes clipped panels", () => {
    const onChange = vi.fn();
    const { container } = render(
      <ForecastScenarioMenuSelect
        value="context"
        options={[
          { value: "context", label: "Contexte" },
          { value: "risk", label: "Risque" },
        ]}
        onChange={onChange}
      />,
    );

    const trigger = screen.getByRole("button", { name: "Contexte" });
    expect(trigger).toHaveClass("btn", "btn-sm", "btn-secondary");
    fireEvent.click(trigger);

    const panel = document.body.querySelector(".fcs-menu-panel");
    expect(panel).not.toBeNull();
    expect(container.contains(panel)).toBe(false);

    fireEvent.click(screen.getByRole("button", { name: "Risque" }));
    expect(onChange).toHaveBeenCalledWith("risk");
  });

  it("renders filters over the chart instead of inside its layout", () => {
    const { container } = render(
      <ForecastViewFilters
        groups={[{
          id: "series",
          titleKey: "forecast.view.filters.series",
          items: [],
          emptyKey: "forecast.view.filters.noLayersYet",
        }]}
        layers={{}}
        onChange={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", {
      name: /forecast\.view\.filters\.button/,
    }));

    const panel = document.body.querySelector(".fcf-panel");
    expect(panel).not.toBeNull();
    expect(container.contains(panel)).toBe(false);
  });
});
