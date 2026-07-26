import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ForecastModelsResponse } from "./forecast-model-meta";
import { useForecastConfigModels } from "./use-forecast-config-models";

const RESPONSE: ForecastModelsResponse = {
  providers: [],
  configured_provider_ids: [],
  models: [{
    id: "chronos-bolt-small",
    provider_id: "amazon",
    display_name: "Chronos Bolt Small",
    params: "21M",
    size_mb: 40,
    ram_mb: 350,
    vram_mb: 120,
    cpu_supported: true,
    gpu_supported: true,
    multivariate: false,
    covariates: false,
    horizon_max: 64,
    frequencies: "D,W,M",
    is_cloud: false,
    installed: true,
    runtime_ready: true,
    runnable: true,
  }],
};

describe("useForecastConfigModels", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockResolvedValue(RESPONSE);
  });

  it("ne réutilise pas silencieusement un modèle Manuel en mode Auto", async () => {
    const { result } = renderHook(() => useForecastConfigModels("", false));

    await waitFor(() => expect(result.current.models).toHaveLength(1));

    expect(result.current.model).toBe("");
  });

  it("masque un modele local tant que son moteur n'est pas pret", async () => {
    vi.mocked(invoke).mockResolvedValue({
      ...RESPONSE,
      models: [{ ...RESPONSE.models[0], runtime_ready: false }],
    });

    const { result } = renderHook(() => useForecastConfigModels("", false));

    await waitFor(() => expect(result.current.models).toHaveLength(0));
    expect(result.current.model).toBe("");
  });
});
