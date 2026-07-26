/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ForecastModelsView } from "../forecast-settings-models";
import type {
  ForecastModelEntry,
  ForecastModelGroup,
  ForecastProviderEntry,
} from "@/components/forecast/forecast-model-meta";
import { forecastModels } from "../test-utils/settings-tab-test-data";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(() => Promise.resolve(null)) }));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/forecast/model-browser/model-specs", () => ({
  ModelSpecs: ({ model, onBack }: { model: { display_name: string }; onBack: () => void }) => (
    <div data-testid="model-specs">
      specs:{model.display_name}
      <button data-testid="specs-back" onClick={onBack}>back</button>
    </div>
  ),
}));

vi.mock("@/components/forecast/model-browser/model-install-btn", () => ({
  ModelInstallBtn: () => <button data-testid="install-btn">install</button>,
}));

function model(overrides: Partial<ForecastModelEntry> = {}): ForecastModelEntry {
  return {
    ...forecastModels().models[0],
    ...overrides,
    runtime_ready: overrides.runtime_ready ?? true,
  };
}

function family(overrides: Partial<ForecastModelGroup> = {}): ForecastModelGroup {
  return {
    id: "chronos-bolt",
    titleKey: "forecast.models.engines.localChronosBolt",
    models: [model()],
    ...overrides,
  };
}

const noop = vi.fn();

describe("ForecastModelsView", () => {
  afterEach(cleanup);

  it("affiche la liste des familles quand rien n'est selectionne", () => {
    const families = [family()];
    render(
      <ForecastModelsView
        families={families}
        providers={[]}
        selectedFamily={null}
        selectedModel={null}
        onSelectFamily={noop}
        onSelectModel={noop}
        onBackToFamily={noop}
        onBackToFamilies={noop}
        onRefresh={noop}
      />,
    );

    expect(screen.getByText("forecast.models.engines.localChronosBolt")).toBeTruthy();
    expect(screen.getByText("1")).toBeTruthy();
  });

  it("affiche l'etat vide s'il n'y a pas de familles", () => {
    render(
      <ForecastModelsView
        families={[]}
        providers={[]}
        selectedFamily={null}
        selectedModel={null}
        onSelectFamily={noop}
        onSelectModel={noop}
        onBackToFamily={noop}
        onBackToFamilies={noop}
        onRefresh={noop}
      />,
    );

    expect(screen.getByText("forecast.models.noneAvailable")).toBeTruthy();
  });

  it("appelle onSelectFamily au clic sur une famille", () => {
    const onSelectFamily = vi.fn();
    render(
      <ForecastModelsView
        families={[family()]}
        providers={[]}
        selectedFamily={null}
        selectedModel={null}
        onSelectFamily={onSelectFamily}
        onSelectModel={noop}
        onBackToFamily={noop}
        onBackToFamilies={noop}
        onRefresh={noop}
      />,
    );

    fireEvent.click(screen.getByText("forecast.models.engines.localChronosBolt"));
    expect(onSelectFamily).toHaveBeenCalledWith("chronos-bolt");
  });

  it("affiche la liste des modeles de la famille selectionnee", () => {
    const { container } = render(
      <ForecastModelsView
        families={[family()]}
        providers={[]}
        selectedFamily={family()}
        selectedModel={null}
        onSelectFamily={noop}
        onSelectModel={noop}
        onBackToFamily={noop}
        onBackToFamilies={noop}
        onRefresh={noop}
      />,
    );

    const nameEl = container.querySelector(".fs-model-name");
    expect(nameEl?.textContent).toBe("Chronos Bolt Small");
    expect(screen.getByText(/small · 120 MB/)).toBeTruthy();
  });

  it("appelle onSelectModel au clic sur un modele", () => {
    const onSelectModel = vi.fn();
    const { container } = render(
      <ForecastModelsView
        families={[family()]}
        providers={[]}
        selectedFamily={family()}
        selectedModel={null}
        onSelectFamily={noop}
        onSelectModel={onSelectModel}
        onBackToFamily={noop}
        onBackToFamilies={noop}
        onRefresh={noop}
      />,
    );

    const modelRow = container.querySelector(".fs-model-row") as HTMLElement;
    fireEvent.click(modelRow);
    expect(onSelectModel).toHaveBeenCalledWith("chronos-bolt-small");
  });

  it("affiche les specs du modele selectionne", () => {
    const providers: ForecastProviderEntry[] = [{ id: "local", display_name: "Local", configured: true }];
    render(
      <ForecastModelsView
        families={[family()]}
        providers={providers}
        selectedFamily={family()}
        selectedModel={model()}
        onSelectFamily={noop}
        onSelectModel={noop}
        onBackToFamily={noop}
        onBackToFamilies={noop}
        onRefresh={noop}
      />,
    );

    expect(screen.getByTestId("model-specs")).toBeTruthy();
    expect(screen.getByTestId("model-specs").textContent).toContain("specs:Chronos Bolt Small");
  });

  it("appelle onBackToFamily au clic sur le bouton retour de la fiche modele", () => {
    const onBackToFamily = vi.fn();
    render(
      <ForecastModelsView
        families={[family()]}
        providers={[]}
        selectedFamily={family()}
        selectedModel={model()}
        onSelectFamily={noop}
        onSelectModel={noop}
        onBackToFamily={onBackToFamily}
        onBackToFamilies={noop}
        onRefresh={noop}
      />,
    );

    fireEvent.click(screen.getByTestId("specs-back"));
    expect(onBackToFamily).toHaveBeenCalledOnce();
  });
});
