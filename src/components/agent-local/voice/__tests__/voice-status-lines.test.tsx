import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { VoiceStatusLines } from "../voice-status-lines";

const dispatch = vi.fn().mockResolvedValue({ revision: 2 });
vi.mock("@/features/voice/voice-client", () => ({
  // Test seam intentionally returns the mock promise.
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  dispatchVoiceAction: (action: unknown) => dispatch(action),
}));
vi.mock("@/features/voice/voice-store", () => ({
  acceptVoiceSnapshot: vi.fn(),
  useVoiceSnapshot: () => null,
}));
vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));

describe("VoiceStatusLines", () => {
  beforeEach(() => dispatch.mockClear());

  it("offers restore and explicit deletion only when recovery is ready", () => {
    render(<VoiceStatusLines draftKey="draft" snapshot={{
      revision: 1, phase: "idle", operation: null, delivery: null, trialResult: null, error: null,
      recovery: { id: "recovery", draftKey: "draft", captureMs: 160_000, status: "ready" },
    }} />);
    fireEvent.click(screen.getByRole("button", { name: "voice.recovery.restore" }));
    expect(dispatch).toHaveBeenCalledWith({ action: "restore-recovery", recovery_id: "recovery", draft_key: "draft" });
    expect(screen.getByRole("button", { name: "voice.recovery.delete" })).toBeVisible();
  });

  it("shows a global recovery in the current draft without an early restore action", () => {
    render(<VoiceStatusLines draftKey="current" snapshot={{
      revision: 1, phase: "recovering", operation: null, delivery: null, trialResult: null, error: null,
      recovery: { id: "recovery", draftKey: null, captureMs: 31_000, status: "preparing" },
    }} />);
    expect(screen.queryByRole("button", { name: "voice.recovery.restore" })).toBeNull();
    expect(screen.getByRole("button", { name: "voice.recovery.delete" })).toBeVisible();
  });
});
