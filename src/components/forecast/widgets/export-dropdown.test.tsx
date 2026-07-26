import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
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
});
