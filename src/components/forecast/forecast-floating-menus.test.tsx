import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { ForecastViewFilters } from "./forecast-view-filters";
import { ForecastNav } from "./forecast-nav";
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

  it("régression: conserve le panneau des filtres monté mais masqué pendant l'inactivité", () => {
    const groups = [{
      id: "series",
      titleKey: "forecast.view.filters.series",
      items: [{ id: "history", label: "Historique", interactive: true }],
    }];
    const view = render(
      <AppSurfaceActivityProvider active>
        <ForecastViewFilters groups={groups} layers={{ history: true }} onChange={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: /forecast\.view\.filters\.button/ }));
    const panel = document.body.querySelector<HTMLElement>(".fcf-panel");
    expect(panel).not.toBeNull();

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <ForecastViewFilters groups={groups} layers={{ history: true }} onChange={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    expect(panel?.isConnected).toBe(true);
    expect(panel?.closest(".app-surface-portal-boundary")).toHaveAttribute("hidden");

    view.rerender(
      <AppSurfaceActivityProvider active>
        <ForecastViewFilters groups={groups} layers={{ history: true }} onChange={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    expect(panel?.closest(".app-surface-portal-boundary")).not.toHaveAttribute("hidden");
  });

  it("cache le menu ouvert quand la surface devient inactive sans perdre son état", () => {
    const view = render(
      <AppSurfaceActivityProvider active>
        <ForecastScenarioMenuSelect
          value="context"
          options={[{ value: "context", label: "Contexte" }, { value: "risk", label: "Risque" }]}
          onChange={vi.fn()}
        />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Contexte" }));
    const panel = document.body.querySelector<HTMLElement>(".fcs-menu-panel");
    expect(panel).not.toBeNull();

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <ForecastScenarioMenuSelect
          value="context"
          options={[{ value: "context", label: "Contexte" }, { value: "risk", label: "Risque" }]}
          onChange={vi.fn()}
        />
      </AppSurfaceActivityProvider>,
    );
    expect(panel?.closest(".app-surface-portal-boundary")).toHaveAttribute("hidden");

    view.rerender(
      <AppSurfaceActivityProvider active>
        <ForecastScenarioMenuSelect
          value="context"
          options={[{ value: "context", label: "Contexte" }, { value: "risk", label: "Risque" }]}
          onChange={vi.fn()}
        />
      </AppSurfaceActivityProvider>,
    );
    expect(panel?.closest(".app-surface-portal-boundary")).not.toHaveAttribute("hidden");
  });

  it("ne refocalise pas le menu scénario resté ouvert au retour", async () => {
    const focus = vi.spyOn(HTMLElement.prototype, "focus");
    let active = true;
    const props = {
      value: "context",
      options: [{ value: "context", label: "Contexte" }, { value: "risk", label: "Risque" }],
      onChange: vi.fn(),
    };
    const view = render(
      <AppSurfaceActivityProvider active={active}>
        <ForecastScenarioMenuSelect {...props} />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Contexte" }));
    await waitFor(() => expect(focus).toHaveBeenCalled());
    const initialCalls = focus.mock.calls.length;

    active = false;
    view.rerender(<AppSurfaceActivityProvider active={active}><ForecastScenarioMenuSelect {...props} /></AppSurfaceActivityProvider>);
    active = true;
    view.rerender(<AppSurfaceActivityProvider active={active}><ForecastScenarioMenuSelect {...props} /></AppSurfaceActivityProvider>);
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    expect(focus.mock.calls.length).toBe(initialCalls);
  });

  it("ne refocalise pas les filtres restés ouverts au retour", async () => {
    const focus = vi.spyOn(HTMLElement.prototype, "focus");
    let active = true;
    const groups = [{
      id: "series",
      titleKey: "forecast.view.filters.series",
      items: [{ id: "history", label: "Historique", interactive: true }],
    }];
    const view = render(
      <AppSurfaceActivityProvider active={active}>
        <ForecastViewFilters groups={groups} layers={{ history: true }} onChange={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: /forecast\.view\.filters\.button/ }));
    await waitFor(() => expect(focus).toHaveBeenCalled());
    const initialCalls = focus.mock.calls.length;

    active = false;
    view.rerender(
      <AppSurfaceActivityProvider active={active}>
        <ForecastViewFilters groups={groups} layers={{ history: true }} onChange={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    active = true;
    view.rerender(
      <AppSurfaceActivityProvider active={active}>
        <ForecastViewFilters groups={groups} layers={{ history: true }} onChange={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    expect(focus.mock.calls.length).toBe(initialCalls);
  });

  it("ne refocalise pas ForecastNav resté ouvert au retour", async () => {
    function Harness({ active }: { active: boolean }) {
      const [open, setOpen] = useState(false);
      return (
        <AppSurfaceActivityProvider active={active}>
          <ForecastNav
            open={open}
            activeSection="view"
            onToggle={() => setOpen(true)}
            onSelect={vi.fn()}
          />
        </AppSurfaceActivityProvider>
      );
    }
    const focus = vi.spyOn(HTMLElement.prototype, "focus");
    const view = render(<Harness active />);
    fireEvent.click(document.querySelector(".fc-nav-trigger") as HTMLElement);
    await waitFor(() => expect(focus).toHaveBeenCalled());
    const initialCalls = focus.mock.calls.length;

    view.rerender(<Harness active={false} />);
    view.rerender(<Harness active />);
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    await waitFor(() => expect(focus.mock.calls.length).toBe(initialCalls));
  });
});
