import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { ExportDropdown } from "./export-dropdown";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => ({
      "forecast.export.title": "Exporter",
      "forecast.export.csv": "CSV",
      "forecast.export.excel": "Excel",
      "forecast.export.png": "PNG",
      "forecast.export.svg": "SVG",
      "forecast.export.json": "JSON",
      "forecast.export.pdf": "PDF",
      "forecast.export.clipboard": "Presse-papiers",
    })[key] ?? key,
  }),
}));

describe("ExportDropdown", () => {
  it("renders outside clipping containers and exports the selected format", () => {
    const onExport = vi.fn();
    const { container } = render(
      <div className="clipping-container">
        <ExportDropdown analysisId="analysis-id" onExport={onExport} />
      </div>,
    );

    fireEvent.click(screen.getByRole("button", { name: "Exporter" }));

    const menu = screen.getByRole("menu");
    expect(container.contains(menu)).toBe(false);
    fireEvent.click(screen.getByRole("button", { name: "CSV" }));
    expect(onExport).toHaveBeenCalledWith("csv", "analysis-id");
  });

  it("conserve le menu ouvert masqué pendant l'inactivité", () => {
    const view = render(
      <AppSurfaceActivityProvider active>
        <ExportDropdown analysisId="analysis-id" onExport={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Exporter" }));
    const menu = document.body.querySelector<HTMLElement>(".exd-menu");
    expect(menu).not.toBeNull();

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <ExportDropdown analysisId="analysis-id" onExport={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    expect(menu?.closest(".app-surface-portal-boundary")).toHaveAttribute("hidden");
  });

  it("ne refocalise pas le menu resté ouvert au retour de surface", async () => {
    const focus = vi.spyOn(HTMLElement.prototype, "focus");
    let active = true;
    const view = render(
      <AppSurfaceActivityProvider active={active}>
        <ExportDropdown analysisId="analysis-id" onExport={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Exporter" }));
    await waitFor(() => expect(focus).toHaveBeenCalled());
    const initialCalls = focus.mock.calls.length;

    active = false;
    view.rerender(
      <AppSurfaceActivityProvider active={active}>
        <ExportDropdown analysisId="analysis-id" onExport={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    active = true;
    view.rerender(
      <AppSurfaceActivityProvider active={active}>
        <ExportDropdown analysisId="analysis-id" onExport={vi.fn()} />
      </AppSurfaceActivityProvider>,
    );
    await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
    expect(focus.mock.calls.length).toBe(initialCalls);
  });
});
