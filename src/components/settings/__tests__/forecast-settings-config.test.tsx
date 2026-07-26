/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ForecastConfigView } from "../forecast-settings-config";
import type { ForecastModelEntry } from "@/components/forecast/forecast-model-meta";
import type { ForecastModelConfig } from "@/components/forecast/model-browser/forecast-config-types";

const invokeMock = vi.fn<(cmd: string, args?: Record<string, unknown>) => Promise<unknown>>();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>): Promise<unknown> => invokeMock(cmd, args),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key },
}));

vi.mock("@/lib/toast-emitter", () => ({
  showToast: vi.fn(),
}));

function model(overrides: Partial<ForecastModelEntry> = {}): ForecastModelEntry {
  return {
    id: "chronos-bolt-small",
    provider_id: "local",
    family_id: "chronos-bolt",
    display_name: "Chronos Bolt Small",
    params: "small",
    size_mb: 120,
    ram_mb: 512,
    vram_mb: null,
    cpu_supported: true,
    gpu_supported: false,
    multivariate: false,
    covariates: false,
    horizon_max: 64,
    frequencies: "D,H",
    is_cloud: false,
    installed: true,
    runnable: true,
    ...overrides,
    runtime_ready: overrides.runtime_ready ?? true,
  };
}

function config(overrides: Partial<ForecastModelConfig> = {}): ForecastModelConfig {
  return {
    model_id: "chronos-bolt-small",
    family_id: "chronos-bolt",
    params: [
      {
        id: "horizon_max_override",
        kind: "integer",
        label_key: "forecast.modelConfig.params.horizon_max_override.label",
        description_key: "forecast.modelConfig.params.horizon_max_override.description",
        default_value: 64,
        value: 64,
        effective_value: 64,
        options: [],
      },
    ],
    inherited: [
      {
        id: "device",
        kind: "select",
        label_key: "forecast.modelConfig.params.device.label",
        description_key: "forecast.modelConfig.params.device.description",
        default_value: "auto",
        value: null,
        effective_value: "cpu",
        options: ["auto", "cpu", "gpu"],
      },
    ],
    ...overrides,
  };
}

describe("ForecastConfigView", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  beforeEach(() => {
    invokeMock.mockResolvedValue(config());
  });

  it("affiche l'etat vide si aucun modele selectionne", () => {
    render(
      <ForecastConfigView models={[]} selectedModel={null} onSelectModel={vi.fn()} />,
    );

    expect(screen.getByText("forecast.modelConfig.noModel")).toBeTruthy();
  });

  it("charge la config du modele selectionne", async () => {
    render(
      <ForecastConfigView
        models={[model()]}
        selectedModel={model()}
        onSelectModel={vi.fn()}
      />,
    );

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("get_forecast_model_config", { modelId: "chronos-bolt-small" });
    });
  });

  it("affiche le bouton edit par defaut", async () => {
    render(
      <ForecastConfigView
        models={[model()]}
        selectedModel={model()}
        onSelectModel={vi.fn()}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("forecast.modelConfig.edit")).toBeTruthy();
    });
  });

  it("bascule en mode edition au clic sur edit", async () => {
    render(
      <ForecastConfigView
        models={[model()]}
        selectedModel={model()}
        onSelectModel={vi.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByText("forecast.modelConfig.edit")).toBeTruthy());
    fireEvent.click(screen.getByText("forecast.modelConfig.edit"));

    expect(screen.getByText("forecast.modelConfig.save")).toBeTruthy();
    expect(screen.getByText("forecast.modelConfig.cancel")).toBeTruthy();
  });

  it("sauvegarde la config au clic sur save", async () => {
    invokeMock.mockResolvedValueOnce(config());
    const saved = config();
    invokeMock.mockResolvedValueOnce(saved);

    render(
      <ForecastConfigView
        models={[model()]}
        selectedModel={model()}
        onSelectModel={vi.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByText("forecast.modelConfig.edit")).toBeTruthy());
    fireEvent.click(screen.getByText("forecast.modelConfig.edit"));
    fireEvent.click(screen.getByText("forecast.modelConfig.save"));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("set_forecast_model_config", expect.objectContaining({
        modelId: "chronos-bolt-small",
      }));
    });
  });

  it("annule l'edition et revient en mode lecture", async () => {
    render(
      <ForecastConfigView
        models={[model()]}
        selectedModel={model()}
        onSelectModel={vi.fn()}
      />,
    );

    await waitFor(() => expect(screen.getByText("forecast.modelConfig.edit")).toBeTruthy());
    fireEvent.click(screen.getByText("forecast.modelConfig.edit"));
    fireEvent.click(screen.getByText("forecast.modelConfig.cancel"));

    expect(screen.getByText("forecast.modelConfig.edit")).toBeTruthy();
  });
});
