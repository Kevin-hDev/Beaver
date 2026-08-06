import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { OllamaModel } from "@/types/agent";
import { OllamaModelfileView } from "../ollama-modelfile-view";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("../modelfile-viewer", () => ({
  ModelfileViewer: ({ modelName, onBack }: { modelName: string; onBack: () => void }) => (
    <div data-testid="modelfile-viewer">
      {modelName}
      <button type="button" aria-label="common.back" onClick={onBack} />
    </div>
  ),
}));

function model(name: string): OllamaModel {
  return { name, size: 0, digest_short: "abc", is_customized: false } as OllamaModel;
}

describe("OllamaModelfileView", () => {
  afterEach(() => cleanup());

  it("liste les modèles installés sans en ouvrir un", () => {
    render(
      <OllamaModelfileView
        models={[model("llama3.2:latest"), model("qwen3:8b")]}
        selected={null}
        onSelect={vi.fn()}
      />,
    );

    expect(screen.getByText("llama3.2:latest")).toBeTruthy();
    expect(screen.queryByTestId("modelfile-viewer")).toBeNull();
  });

  it("ouvre la fiche du modèle choisi", () => {
    const onSelect = vi.fn();
    render(
      <OllamaModelfileView models={[model("qwen3:8b")]} selected={null} onSelect={onSelect} />,
    );

    fireEvent.click(screen.getByText("qwen3:8b"));

    expect(onSelect).toHaveBeenCalledWith("qwen3:8b");
  });

  it("revient à la liste depuis la fiche", () => {
    const onSelect = vi.fn();
    render(
      <OllamaModelfileView
        models={[model("qwen3:8b")]}
        selected="qwen3:8b"
        onSelect={onSelect}
      />,
    );

    expect(screen.getByTestId("modelfile-viewer").textContent).toBe("qwen3:8b");
    fireEvent.click(screen.getByLabelText("common.back"));

    expect(onSelect).toHaveBeenCalledWith(null);
  });

  it("annonce l'absence de modèle plutôt qu'une bulle vide", () => {
    render(<OllamaModelfileView models={[]} selected={null} onSelect={vi.fn()} />);

    expect(screen.getByText("ollama.noInstalledModels")).toBeTruthy();
  });
});
