import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import type { ReactElement } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ModelfileEditor } from "../modelfile-editor";
import { ParametersEditor } from "../parameters-editor";
import { SystemPromptEditor } from "../system-prompt-editor";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const modelName = "llama3.2:latest";

describe("Ollama model editors", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset().mockResolvedValue(undefined);
  });

  it.each(editorCases())("affiche %s dans une carte centrée", (_name, createEditor, fillsSpace) => {
    const { container } = render(createEditor());

    const page = container.querySelector(".mes-page");
    const inner = page?.querySelector(":scope > .mes-inner");
    const card = inner?.querySelector(":scope > .settings-card.mes-card");

    expect(page).toBeTruthy();
    expect(inner).toBeTruthy();
    expect(card).toBeTruthy();
    expect(page).toHaveClass(fillsSpace ? "mes-page-fill" : "mes-page");
    if (!fillsSpace) expect(page).not.toHaveClass("mes-page-fill");
  });

  it.each(["dark", "light"])("conserve la carte avec le thème %s", (theme) => {
    const { container } = render(
      <div data-theme={theme}>
        <SystemPromptEditor
          modelName={modelName}
          initialSystem=""
          onSave={vi.fn()}
          onCancel={vi.fn()}
        />
      </div>,
    );

    expect(container.querySelector(`[data-theme="${theme}"] .settings-card.mes-card`)).toBeTruthy();
  });

  it("sauvegarde le system prompt depuis la carte", async () => {
    const onSave = vi.fn();
    render(
      <SystemPromptEditor
        modelName={modelName}
        initialSystem="Ancien prompt"
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByRole("textbox", { name: "ollama.systemPrompt" }), {
      target: { value: "Nouveau prompt" },
    });
    fireEvent.click(screen.getByText("ollama.save"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("update_system_prompt", {
        name: modelName,
        system: "Nouveau prompt",
      });
      expect(onSave).toHaveBeenCalledWith("Nouveau prompt");
    });
  });

  it("sauvegarde le modelfile depuis la carte", async () => {
    const onSave = vi.fn();
    render(
      <ModelfileEditor
        modelName={modelName}
        initialContent="FROM llama3.2"
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    fireEvent.change(screen.getByRole("textbox", { name: "ollama.editing" }), {
      target: { value: "FROM llama3.3" },
    });
    fireEvent.click(screen.getByText("ollama.save"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("update_modelfile", {
        name: modelName,
        content: "FROM llama3.3",
      });
      expect(onSave).toHaveBeenCalledWith("FROM llama3.3");
    });
  });

  it("affiche une erreur générique si le modelfile ne peut pas être sauvegardé", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("internal path"));
    render(
      <ModelfileEditor
        modelName={modelName}
        initialContent="FROM llama3.2"
        onSave={vi.fn()}
        onCancel={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByText("ollama.save"));

    expect(await screen.findByRole("alert")).toHaveTextContent("errors.operationFailed");
  });

  it("sauvegarde les paramètres depuis la carte", async () => {
    const onSave = vi.fn();
    render(
      <ParametersEditor
        modelName={modelName}
        initialParameters={[{ key: "temperature", value: "0.7" }]}
        onSave={onSave}
        onCancel={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByText("ollama.save"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("update_parameters", {
        name: modelName,
        parameters: [["temperature", "0.7"]],
      });
      expect(onSave).toHaveBeenCalledOnce();
    });
  });
});

function editorCases(): Array<[string, () => ReactElement, boolean]> {
  return [
    [
      "le system prompt",
      () => (
        <SystemPromptEditor
          modelName={modelName}
          initialSystem=""
          onSave={vi.fn()}
          onCancel={vi.fn()}
        />
      ),
      true,
    ],
    [
      "les paramètres",
      () => (
        <ParametersEditor
          modelName={modelName}
          initialParameters={[]}
          onSave={vi.fn()}
          onCancel={vi.fn()}
        />
      ),
      false,
    ],
    [
      "le modelfile",
      () => (
        <ModelfileEditor
          modelName={modelName}
          initialContent="FROM llama3.2"
          onSave={vi.fn()}
          onCancel={vi.fn()}
        />
      ),
      true,
    ],
  ];
}
