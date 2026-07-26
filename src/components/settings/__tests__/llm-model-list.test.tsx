import { cleanup, render, fireEvent } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { LlmModelList } from "../llm-model-list";
import type { RegistryModelInfo } from "../llm-types";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key, language: "fr" },
}));

afterEach(cleanup);

function model(overrides: Partial<RegistryModelInfo> = {}): RegistryModelInfo {
  return {
    key: "gpt-4",
    provider: "openai",
    mode: "chat",
    max_input_tokens: 128000,
    max_output_tokens: 4096,
    input_cost_per_mtok: 5.0,
    output_cost_per_mtok: 15.0,
    supports_vision: true,
    supports_function_calling: true,
    supports_reasoning: false,
    supports_prompt_caching: false,
    supports_audio_input: false,
    supports_audio_output: false,
    supports_web_search: false,
    supports_response_schema: false,
    supports_system_messages: true,
    ...overrides,
  };
}

describe("LlmModelList", () => {
  it("affiche le titre et le compte de modèles", () => {
    const models = [model(), model({ key: "gpt-3.5" })];
    const { getByText } = render(
      <LlmModelList models={models} title="OpenAI" onSelect={() => {}} />,
    );

    expect(getByText(/OpenAI/)).toBeTruthy();
    expect(getByText("(2)")).toBeTruthy();
  });

  it("rend une ligne par modèle", () => {
    const models = [model(), model({ key: "claude-3" }), model({ key: "mistral" })];
    const { getByText } = render(
      <LlmModelList models={models} title="Test" onSelect={() => {}} />,
    );

    expect(getByText("gpt-4")).toBeTruthy();
    expect(getByText("claude-3")).toBeTruthy();
    expect(getByText("mistral")).toBeTruthy();
  });

  it("appelle onSelect au clic sur une ligne", () => {
    const onSelect = vi.fn();
    const m = model();
    const { getByText } = render(
      <LlmModelList models={[m]} title="Test" onSelect={onSelect} />,
    );

    fireEvent.click(getByText("gpt-4"));

    expect(onSelect).toHaveBeenCalledOnce();
    expect(onSelect).toHaveBeenCalledWith(m);
  });

  it("appelle onSelect avec la touche Enter", () => {
    const onSelect = vi.fn();
    const m = model();
    const { getByText } = render(
      <LlmModelList models={[m]} title="Test" onSelect={onSelect} />,
    );

    const row = getByText("gpt-4").closest("[role=button]")!;
    fireEvent.keyDown(row, { key: "Enter" });

    expect(onSelect).toHaveBeenCalledOnce();
  });

  it("appelle onSelect avec la touche Espace", () => {
    const onSelect = vi.fn();
    const m = model();
    const { getByText } = render(
      <LlmModelList models={[m]} title="Test" onSelect={onSelect} />,
    );

    const row = getByText("gpt-4").closest("[role=button]")!;
    fireEvent.keyDown(row, { key: " " });

    expect(onSelect).toHaveBeenCalledOnce();
  });

  it("n'appelle pas onSelect avec une autre touche", () => {
    const onSelect = vi.fn();
    const m = model();
    const { getByText } = render(
      <LlmModelList models={[m]} title="Test" onSelect={onSelect} />,
    );

    const row = getByText("gpt-4").closest("[role=button]")!;
    fireEvent.keyDown(row, { key: "a" });

    expect(onSelect).not.toHaveBeenCalled();
  });

  it("affiche le bouton retour seulement si onBack est fourni", () => {
    const { rerender, queryByText, getByText } = render(
      <LlmModelList models={[]} title="Test" onSelect={() => {}} />,
    );

    expect(queryByText(/back/)).toBeNull();

    rerender(
      <LlmModelList models={[]} title="Test" onSelect={() => {}} onBack={() => {}} />,
    );

    expect(getByText(/settings.llm.back/)).toBeTruthy();
  });

  it("appelle onBack au clic du bouton retour", () => {
    const onBack = vi.fn();
    const { getByText } = render(
      <LlmModelList models={[]} title="Test" onSelect={() => {}} onBack={onBack} />,
    );

    fireEvent.click(getByText(/settings.llm.back/));

    expect(onBack).toHaveBeenCalledOnce();
  });

  it("affiche le coût par million de tokens si présent", () => {
    const m = model({ input_cost_per_mtok: 2.5 });
    const { getByText } = render(
      <LlmModelList models={[m]} title="Test" onSelect={() => {}} />,
    );

    // 2.50/M (toFixed(2))
    expect(getByText("$2.50/M")).toBeTruthy();
  });

  it("n'affiche pas le coût si input_cost_per_mtok est null", () => {
    const m = model({ input_cost_per_mtok: null });
    const { container } = render(
      <LlmModelList models={[m]} title="Test" onSelect={() => {}} />,
    );

    expect(container.textContent).not.toContain("/M");
  });

  it("affiche le contexte en K si max_input_tokens présent", () => {
    const m = model({ max_input_tokens: 128000 });
    const { getByText } = render(
      <LlmModelList models={[m]} title="Test" onSelect={() => {}} />,
    );

    // 128000 / 1000 = 128 → "128K ctx"
    expect(getByText(/128K ctx/)).toBeTruthy();
  });
});
