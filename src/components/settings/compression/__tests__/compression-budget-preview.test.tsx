import { render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
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
    before_tokens: 96_000,
    system_tools_tokens: 12_000,
    variable_tokens: 16_800,
    target_tokens: 28_800,
    range_lower_tokens: 24_000,
    range_upper_tokens: 32_000,
    image_count: 4,
    reduction_lower_percent: 67,
    reduction_upper_percent: 75,
  })),
}));

const projection = () => ({
  before_tokens: 96_000,
  system_tools_tokens: 12_000,
  variable_tokens: 16_800,
  target_tokens: 28_800,
  range_lower_tokens: 24_000,
  range_upper_tokens: 32_000,
  image_count: 4,
  reduction_lower_percent: 67,
  reduction_upper_percent: 75,
});

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(invoke).mockResolvedValue(projection());
});

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
    vi.mocked(invoke).mockResolvedValueOnce(projection());
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

  it("vide l'ancienne projection pendant le chargement d'une autre plage", async () => {
    const { rerender } = render(
      <CompressionBudgetPreview profileId="beaver" profileRevision={1} band="compact" />,
    );
    await screen.findByText("28.8K");
    vi.mocked(invoke).mockImplementationOnce(() => new Promise(() => {}));

    rerender(
      <CompressionBudgetPreview profileId="beaver" profileRevision={1} band="large" />,
    );

    await screen.findByText("settings.advanced.compressionProjectionLoading");
    expect(screen.queryByText("28.8K")).not.toBeInTheDocument();
  });

  it("quitte l'état d'échec quand une nouvelle projection réussit", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("failed"));
    const { rerender } = render(
      <CompressionBudgetPreview profileId="beaver" profileRevision={1} band="compact" />,
    );
    await screen.findByText("settings.advanced.compressionProjectionUnavailable");
    vi.mocked(invoke).mockResolvedValueOnce(projection());

    rerender(
      <CompressionBudgetPreview profileId="beaver" profileRevision={2} band="compact" />,
    );

    await screen.findByText("settings.advanced.compressionProjectionTarget");
    expect(screen.queryByText("settings.advanced.compressionProjectionUnavailable"))
      .not.toBeInTheDocument();
  });
});
