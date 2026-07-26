import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ModelSelector } from "../model-selector";
import type { AvailableModel } from "@/hooks/use-available-models";

let groups = new Map<string, AvailableModel[]>();

vi.mock("@/hooks/use-available-models", () => ({
  useAvailableModels: () => ({ groups }),
}));

vi.mock("@/hooks/use-favorite-models", () => ({
  useFavoriteModels: () => ({
    favorites: [],
    isFavorite: () => false,
    toggle: vi.fn(),
  }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const labels: Record<string, string> = {
        "agentLocal.selectModel": "Select model",
        "agentLocal.reasoningAuto": "Activée",
        "agentLocal.reasoningHigh": "Forte",
      };
      return labels[key] ?? key;
    },
  }),
}));

afterEach(() => {
  cleanup();
  groups = new Map();
});

function model(overrides: Partial<AvailableModel>): AvailableModel {
  return {
    id: "llama3",
    provider_id: "ollama",
    provider_name: "Ollama",
    auth_source: "local",
    is_local: true,
    supports_tools: false,
    supports_thinking: true,
    ...overrides,
  };
}

describe("ModelSelector", () => {
  it("n'affiche plus la réflexion dans le sélecteur de modèle", () => {
    groups = new Map([["ollama", [model({ id: "llama3" })]]]);

    const { container } = render(
      <ModelSelector
        groups={groups}
        selectedModel="llama3"
        selectedProvider="ollama"
        onSelect={vi.fn()}
      />,
    );

    expect(screen.getByText("llama3")).toBeTruthy();
    expect(screen.queryByText("Activée")).toBeNull();
    expect(container.querySelector(".ms-trigger-reasoning-icon")).toBeNull();
  });

  it("réserve le bouton au nom du modèle", () => {
    groups = new Map([["openai", [model({
      id: "gpt-5",
      provider_id: "openai",
      provider_name: "OpenAI",
      is_local: false,
    })]]]);

    const { container } = render(
      <ModelSelector
        groups={groups}
        selectedModel="gpt-5"
        selectedProvider="openai"
        onSelect={vi.fn()}
      />,
    );

    expect(screen.getByText("gpt-5")).toBeTruthy();
    expect(screen.queryByText("Forte")).toBeNull();
    expect(container.querySelector(".ms-trigger-reasoning-icon")).toBeNull();
  });

  it("ouvre la liste des modèles dans un portail global", () => {
    groups = new Map([["ollama", [model({ id: "llama3" })]]]);

    const { container } = render(
      <div data-testid="host">
        <ModelSelector
          groups={groups}
          selectedModel="llama3"
          selectedProvider="ollama"
          onSelect={vi.fn()}
        />
      </div>,
    );

    fireEvent.click(screen.getByText("llama3"));

    expect(document.body.querySelector(".ms-dropdown")).toBeTruthy();
    expect(container.querySelector(".ms-dropdown")).toBeNull();
  });

  it("affiche le nom officiel mais sélectionne l'identifiant technique", () => {
    const onSelect = vi.fn();
    groups = new Map([["moonshot-oauth", [model({
      id: "kimi-for-coding",
      display_name: "K2.7 Coding",
      provider_id: "moonshot-oauth",
      provider_name: "Moonshot AI · OAuth",
      is_local: false,
    })]]]);

    render(
      <ModelSelector
        groups={groups}
        selectedModel="kimi-for-coding"
        selectedProvider="moonshot-oauth"
        onSelect={onSelect}
      />,
    );

    fireEvent.click(screen.getByText("K2.7 Coding"));
    fireEvent.click(screen.getAllByText("K2.7 Coding")[1]);

    expect(onSelect).toHaveBeenCalledWith("kimi-for-coding", "moonshot-oauth");
  });

  it("affiche un modèle indisponible sans permettre sa sélection", () => {
    const onSelect = vi.fn();
    groups = new Map([["forecast", [model({
      id: "chronos",
      display_name: "Chronos",
      disabled: true,
      disabled_hint: "Mise à jour requise",
    })]]]);

    render(
      <ModelSelector
        groups={groups}
        selectedModel=""
        selectedProvider=""
        onSelect={onSelect}
      />,
    );

    fireEvent.click(screen.getByText("Select model"));
    const item = screen.getByText("Chronos").closest(".ms-item");
    expect(item?.getAttribute("aria-disabled")).toBe("true");
    expect(screen.getByText("Mise à jour requise")).toBeTruthy();
    fireEvent.click(screen.getByText("Chronos"));
    expect(onSelect).not.toHaveBeenCalled();
  });
});
