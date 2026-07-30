import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OllamaModelsView, type OllamaSearchState } from "../ollama-models-view";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("../model-search", () => ({
  ModelSearch: () => <div data-testid="model-search" />,
}));

vi.mock("../model-variants-list", () => ({
  ModelVariantsList: ({ familyName }: { familyName: string }) => (
    <div data-testid="variants">{familyName}</div>
  ),
}));

vi.mock("../model-profile", () => ({
  ModelProfile: ({ variantFullName }: { variantFullName: string | null }) => (
    <div data-testid="profile">{variantFullName}</div>
  ),
}));

const search: OllamaSearchState = {
  query: "",
  setQuery: vi.fn(),
  results: [],
  setResults: vi.fn(),
  searching: false,
  setSearching: vi.fn(),
};

describe("OllamaModelsView", () => {
  afterEach(() => cleanup());

  it("part de la recherche quand aucune famille n'est ouverte", () => {
    render(
      <OllamaModelsView
        search={search}
        family={null}
        variant={null}
        onSelectFamily={vi.fn()}
        onSelectVariant={vi.fn()}
      />,
    );

    expect(screen.getByTestId("model-search")).toBeTruthy();
    expect(screen.queryByLabelText("common.back")).toBeNull();
  });

  it("affiche les variantes d'une famille avec un retour vers la recherche", () => {
    const onSelectFamily = vi.fn();
    render(
      <OllamaModelsView
        search={search}
        family="qwen3"
        variant={null}
        onSelectFamily={onSelectFamily}
        onSelectVariant={vi.fn()}
      />,
    );

    expect(screen.getByTestId("variants").textContent).toBe("qwen3");
    fireEvent.click(screen.getByLabelText("common.back"));

    expect(onSelectFamily).toHaveBeenCalledWith(null);
  });

  it("revient aux variantes depuis le profil, pas à la recherche", () => {
    const onSelectFamily = vi.fn();
    const onSelectVariant = vi.fn();
    render(
      <OllamaModelsView
        search={search}
        family="qwen3"
        variant="qwen3:8b"
        onSelectFamily={onSelectFamily}
        onSelectVariant={onSelectVariant}
      />,
    );

    expect(screen.getByTestId("profile").textContent).toBe("qwen3:8b");
    fireEvent.click(screen.getByLabelText("common.back"));

    expect(onSelectVariant).toHaveBeenCalledWith(null);
    expect(onSelectFamily).not.toHaveBeenCalled();
  });
});
