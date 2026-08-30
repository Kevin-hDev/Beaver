import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import {
  CompressionBudgetPreview,
  formatCompressionWindow,
} from "../compression-budget-preview";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve({
    context_window: 72_345,
    band: "compact",
    system_tools_tokens: 12_000,
    summary_tokens: 5_000,
    categories_tokens: 20_000,
    reserve_tokens: 4_000,
    total_tokens: 41_000,
    projected_percent: 56,
    exceeds_window: false,
    high_risk: true,
  })),
}));

describe("CompressionBudgetPreview", () => {
  it("affiche 1M et accepte une fenêtre valide absente des boutons", async () => {
    render(
      <CompressionBudgetPreview
        profileId="beaver"
        profileRevision={1}
        band="compact"
        currentWindow={72_345}
      />,
    );
    expect(formatCompressionWindow(1_000_000)).toBe("1M");
    await waitFor(() => expect(invoke).toHaveBeenCalledWith(
      "project_settings_compression_budget",
      { profileId: "beaver", band: "compact", contextWindow: 72_345 },
    ));
    expect(screen.getByText("settings.advanced.compressionProjectionTotal")).toBeInTheDocument();
  });

  it("rend le dépassement et le repère de fenêtre visibles", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      context_window: 64_000,
      band: "compact",
      system_tools_tokens: 20_000,
      summary_tokens: 20_000,
      categories_tokens: 30_000,
      reserve_tokens: 10_000,
      total_tokens: 80_000,
      projected_percent: 125,
      exceeds_window: true,
      high_risk: true,
    });
    const { container } = render(
      <CompressionBudgetPreview
        profileId="beaver"
        profileRevision={1}
        band="compact"
        currentWindow={64_000}
      />,
    );

    await waitFor(() => expect(container.querySelector(".cbp-gauge-over")).toBeInTheDocument());
    expect(container.querySelector(".cbp-gauge")).toHaveAttribute("data-overflow", "true");
    expect(container.querySelector(".cbp-gauge-limit")).toHaveStyle({ left: "80%" });
  });
});
