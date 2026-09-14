import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { VoiceProbeGate } from "../voice-probe-panel";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

const idle = { status: "idle", sample_count: 0, levels: [] };
const listening = { status: "listening", sample_count: 128, levels: [0.25] };

describe("VoiceProbeGate", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(listen).mockReset().mockResolvedValue(vi.fn());
  });

  it("monte la sonde après la sentinelle sans démarrer le microphone", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(true);

    render(<VoiceProbeGate />);

    expect(await screen.findByRole("region", { name: "voiceProbe.title" })).toBeInTheDocument();
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("voice_probe_available");
  });

  it("reste absent si la commande n'existe pas ou échoue", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("/private/path"));

    render(<VoiceProbeGate />);

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("voice_probe_available"));
    expect(screen.queryByRole("region", { name: "voiceProbe.title" })).toBeNull();
    expect(screen.queryByText("/private/path")).toBeNull();
  });

  it("ignore une réponse de disponibilité après démontage", async () => {
    const pending = deferred<boolean>();
    vi.mocked(invoke).mockReturnValueOnce(pending.promise);
    const view = render(<VoiceProbeGate />);

    view.unmount();
    await act(async () => {
      pending.resolve(true);
      await pending.promise;
    });

    expect(screen.queryByRole("region", { name: "voiceProbe.title" })).toBeNull();
  });

  it("ignore un relevé de niveau non borné", async () => {
    let receive!: (event: { payload: unknown }) => void;
    vi.mocked(invoke).mockResolvedValueOnce(true);
    vi.mocked(listen).mockImplementationOnce((_event, handler) => {
      receive = handler as (event: { payload: unknown }) => void;
      return Promise.resolve(vi.fn());
    });
    render(<VoiceProbeGate />);
    await screen.findByRole("region", { name: "voiceProbe.title" });

    act(() => receive({
      payload: { status: "listening", sample_count: 1, levels: Array(101).fill(1) },
    }));

    expect(screen.getByRole("progressbar", { name: "voiceProbe.level" }))
      .toHaveAttribute("aria-valuenow", "0");
  });

  it("refuse un second démarrage pendant le premier", async () => {
    const pending = deferred<typeof listening>();
    vi.mocked(invoke)
      .mockResolvedValueOnce(true)
      .mockReturnValueOnce(pending.promise);
    render(<VoiceProbeGate />);
    const start = await screen.findByRole("button", { name: "voiceProbe.start" });

    fireEvent.click(start);
    fireEvent.click(start);

    expect(vi.mocked(invoke).mock.calls.filter(([command]) => command === "voice_probe_start"))
      .toHaveLength(1);
    await act(async () => {
      pending.resolve(listening);
      await pending.promise;
    });
  });

  it("permet l'arrêt après un échec de démarrage", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(true)
      .mockRejectedValueOnce(new Error("microphone unavailable"))
      .mockResolvedValueOnce(idle);
    render(<VoiceProbeGate />);
    fireEvent.click(await screen.findByRole("button", { name: "voiceProbe.start" }));

    fireEvent.click(await screen.findByRole("button", { name: "voiceProbe.stop" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("voice_probe_stop"));
  });
});
