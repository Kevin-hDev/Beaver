import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { VoiceControls } from "../voice-controls";

const controller = vi.fn();
vi.mock("@/features/voice/use-voice-controller", () => ({
  // Test seam intentionally mirrors the hook's structural return value.
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  useVoiceController: () => controller(),
}));
vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));

const idle = {
  available: true, origin: false, activeElsewhere: false, pending: false, dialog: null,
  snapshot: { phase: "idle", operation: null }, begin: vi.fn(), acceptExplanation: vi.fn(),
  closeDialog: vi.fn(), start: vi.fn(), validate: vi.fn(), cancel: vi.fn(),
  modelDownload: null, cancelDownload: vi.fn(), resumeDownload: vi.fn(),
};

describe("VoiceControls", () => {
  beforeEach(() => controller.mockReturnValue(idle));

  it("shows the microphone only when available and idle", () => {
    const { rerender } = render(<VoiceControls draftKey="session:one" />);
    expect(screen.getByRole("button", { name: "voice.start" })).toBeVisible();
    controller.mockReturnValue({ ...idle, available: false });
    rerender(<VoiceControls draftKey="session:one" />);
    expect(screen.queryByRole("button", { name: "voice.start" })).toBeNull();
  });

  it("replaces the microphone with stable listening controls", () => {
    controller.mockReturnValue({ ...idle, origin: true, snapshot: { phase: "listening", operation: { id: "op", level: 0.7 } } });
    render(<VoiceControls draftKey="session:one" />);
    expect(screen.getByRole("button", { name: "voice.cancel" })).toBeVisible();
    expect(screen.getByRole("button", { name: "voice.validate" })).toBeVisible();
    expect(screen.getByRole("img", { name: "voice.status.listening" }).querySelector("polygon")).toHaveStyle({ transform: "scaleY(0.7)" });
    expect(screen.queryByRole("button", { name: "voice.start" })).toBeNull();
  });

  it("leaves active model download progress to the global window", () => {
    controller.mockReturnValue({ ...idle, modelDownload: { id: "download", status: "running", percent: 42 } });
    const { container } = render(<VoiceControls draftKey="session:one" />);
    expect(container).toBeEmptyDOMElement();
    expect(screen.queryByRole("button", { name: "voice.start" })).toBeNull();
  });

  it("can resume a suspended voice model download", () => {
    const resumeDownload = vi.fn();
    controller.mockReturnValue({ ...idle, resumeDownload, modelDownload: { id: "download", status: "suspended", percent: 42 } });
    render(<VoiceControls draftKey="session:one" />);
    screen.getByRole("button", { name: "modelDownloads.resume" }).click();
    expect(resumeDownload).toHaveBeenCalledWith("download");
  });

  it("uses the standard Beaver button geometry in the first-use dialog", () => {
    controller.mockReturnValue({ ...idle, dialog: "first-use" });
    render(<VoiceControls draftKey="session:one" />);
    expect(screen.getByRole("button", { name: "voice.firstUse.later" })).toHaveClass("btn-sm");
    expect(screen.getByRole("button", { name: "voice.firstUse.continue" })).toHaveClass("btn-sm");
  });
});
