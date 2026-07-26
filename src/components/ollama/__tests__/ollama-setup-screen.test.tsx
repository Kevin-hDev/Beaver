import { cleanup, fireEvent, render, screen, waitFor, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { OllamaSetupScreen } from "../ollama-setup-screen";

interface ProgressEvent {
  completed: number;
  total: number;
  status: string;
}

let activeChannel: { onmessage?: (event: ProgressEvent) => void };

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  Channel: function MockChannel() {
    const channel = {};
    activeChannel = channel;
    return channel;
  },
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("OllamaSetupScreen", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => cleanup());

  it("affiche le telechargement puis la phase installation", async () => {
    vi.mocked(invoke).mockImplementation(() => new Promise(() => {}));

    render(<OllamaSetupScreen onComplete={vi.fn()} />);
    fireEvent.click(screen.getByText("ollamaSetup.download"));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("download_ollama", expect.any(Object)));

    act(() => activeChannel.onmessage?.({ completed: 50, total: 100, status: "downloading" }));
    expect(screen.getByText("ollamaSetup.downloading 50%")).toBeTruthy();

    act(() => activeChannel.onmessage?.({ completed: 0, total: 0, status: "extracting" }));
    expect(screen.getByText("ollamaSetup.extracting")).toBeTruthy();
    expect(document.querySelector(".oss-progress-fill-indeterminate")).toBeTruthy();
  });

  it("demande l'annulation du setup en cours", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "cancel_ollama_setup") return Promise.resolve();
      return new Promise(() => {});
    });

    render(<OllamaSetupScreen onComplete={vi.fn()} />);
    fireEvent.click(screen.getByText("ollamaSetup.download"));

    await waitFor(() => expect(screen.getByText("ollamaSetup.cancel")).toBeTruthy());
    fireEvent.click(screen.getByText("ollamaSetup.cancel"));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("cancel_ollama_setup"));
    await waitFor(() => expect(screen.getByText("ollamaSetup.download")).toBeTruthy());
    expect(screen.queryByText("ollamaSetup.cancelling")).toBeNull();
  });
});
