import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ModelfileViewer } from "../modelfile-viewer";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const modelName = "gemma4:e2b";

describe("Ollama system prompt warning", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset().mockImplementation((command) => {
      if (command === "get_modelfile") return Promise.resolve("FROM gemma4:e2b\n");
      return Promise.resolve(undefined);
    });
  });

  it("avertit avant d’ouvrir l’éditeur", async () => {
    render(<ModelfileViewer modelName={modelName} />);
    await screen.findByRole("heading", { name: modelName });

    fireEvent.click(screen.getAllByRole("button", { name: "ollama.edit" })[0]);

    expect(screen.getByRole("dialog", { name: "ollama.systemPromptWarningTitle" })).toBeVisible();
    expect(screen.queryByRole("textbox", { name: "ollama.systemPrompt" })).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "ollama.systemPromptWarningContinue" }));
    expect(screen.getByRole("textbox", { name: "ollama.systemPrompt" })).toBeVisible();
  });

  it("mémorise le choix uniquement quand la case est cochée", async () => {
    const first = render(<ModelfileViewer modelName={modelName} />);
    await screen.findByRole("heading", { name: modelName });
    fireEvent.click(screen.getAllByRole("button", { name: "ollama.edit" })[0]);
    fireEvent.click(screen.getByLabelText("ollama.systemPromptWarningRemember"));
    fireEvent.click(screen.getByRole("button", { name: "ollama.systemPromptWarningContinue" }));

    expect(localStorage.getItem("ollama-system-prompt-warning-dismissed-v1")).toBe("1");
    first.unmount();

    render(<ModelfileViewer modelName={modelName} />);
    await screen.findByRole("heading", { name: modelName });
    fireEvent.click(screen.getAllByRole("button", { name: "ollama.edit" })[0]);

    await waitFor(() => {
      expect(screen.queryByRole("dialog")).toBeNull();
      expect(screen.getByRole("textbox", { name: "ollama.systemPrompt" })).toBeVisible();
    });
  });
});
