import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { OllamaSettingsSection } from "./ollama-settings-section";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@/hooks/use-ollama-runtime-status", () => ({
  useOllamaRuntimeStatus: () => ({ refresh: vi.fn().mockResolvedValue(undefined) }),
}));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("OllamaSettingsSection", () => {
  it("waits for the CPU setting to be persisted before restarting Ollama", async () => {
    let finishSave: ((saved: boolean) => void) | undefined;
    const save = new Promise<boolean>((resolve) => { finishSave = resolve; });
    const onSave = vi.fn().mockReturnValue(save);
    vi.mocked(invoke).mockResolvedValue({ owned_started: { endpoint: { port: 11_434 } } });

    render(
      <OllamaSettingsSection
        keepAlive="5m"
        hardwareAccel="gpu"
        multiModel={false}
        showGpuStatus={true}
        onSave={onSave}
      />,
    );

    fireEvent.click(screen.getByText("settings.advanced.hardwareAccelGpu"));
    fireEvent.click(screen.getByText("settings.advanced.hardwareAccelCpu"));
    fireEvent.click(screen.getAllByText("settings.advanced.hardwareAccelRestart")[0]);

    expect(onSave).toHaveBeenCalledWith({ hardware_accel: "cpu" });
    expect(invoke).not.toHaveBeenCalledWith("restart_ollama_sidecar");

    finishSave?.(true);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("restart_ollama_sidecar");
    });
  });
});
