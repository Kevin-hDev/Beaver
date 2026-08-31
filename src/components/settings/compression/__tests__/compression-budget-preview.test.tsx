import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import {
  CompressionBudgetPreview,
  formatCompressionWindow,
} from "../compression-budget-preview";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, number>) => (
      key === "settings.advanced.compressionProjectionReduction"
        ? `${key}:${values?.minimum}-${values?.maximum}`
        : key
    ),
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve({
    band: "compact",
    before_tokens: 96_000,
    system_tools_tokens: 12_000,
    variable_tokens: 16_800,
    target_tokens: 28_800,
    range_lower_tokens: 24_000,
    range_upper_tokens: 32_000,
    image_count: 4,
    projected_percent: 30,
    reduction_lower_percent: 67,
    reduction_upper_percent: 75,
  })),
}));

describe("CompressionBudgetPreview", () => {
  it("affiche 1M et utilise uniquement la démonstration backend fixe", async () => {
    render(
      <CompressionBudgetPreview
        profileId="beaver"
        profileRevision={1}
        band="compact"
      />,
    );
    expect(formatCompressionWindow(1_000_000)).toBe("1M");
    await waitFor(() => expect(invoke).toHaveBeenCalledWith(
      "project_settings_compression_budget",
      { profileId: "beaver", band: "compact" },
    ));
    expect(screen.getByText("settings.advanced.compressionProjectionTarget")).toBeInTheDocument();
  });

  it("affiche la tranche cible, les images et la note sur le tour actif", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      band: "compact",
      before_tokens: 96_000,
      system_tools_tokens: 12_000,
      variable_tokens: 16_800,
      target_tokens: 28_800,
      range_lower_tokens: 24_000,
      range_upper_tokens: 32_000,
      image_count: 4,
      projected_percent: 30,
      reduction_lower_percent: 67,
      reduction_upper_percent: 75,
    });
    const { container } = render(
      <CompressionBudgetPreview
        profileId="beaver"
        profileRevision={1}
        band="compact"
      />,
    );

    await waitFor(() => expect(screen.getByText("28.8K")).toBeInTheDocument());
    expect(container.querySelector(".cbp-gauge-profile")).toHaveStyle({ width: "17.5%" });
    expect(screen.getByText("settings.advanced.compressionProjectionImages")).toBeInTheDocument();
    expect(screen.getByText("settings.advanced.compressionProjectionActiveTurn")).toBeInTheDocument();
    expect(screen.getByText(
      "settings.advanced.compressionProjectionReduction:67-75",
    )).toBeInTheDocument();
  });
});
