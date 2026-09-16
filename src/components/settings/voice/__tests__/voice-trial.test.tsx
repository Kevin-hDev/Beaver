import { render } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { VoiceTrial } from "../voice-trial";

const dispatch = vi.hoisted(() => vi.fn());

vi.mock("@/features/voice/voice-client", () => ({ dispatchVoiceAction: dispatch }));
vi.mock("@/features/voice/voice-store", () => ({ useVoiceSnapshot: () => null }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key, i18n: { language: "fr" } }),
}));

describe("VoiceTrial", () => {
  beforeEach(() => {
    dispatch.mockReset().mockResolvedValue(null);
    vi.spyOn(crypto, "randomUUID").mockReturnValue("00000000-0000-4000-8000-000000000001");
  });

  it("releases an active trial when the panel closes", () => {
    const view = render(<VoiceTrial
      language={{ kind: "automatic" }}
      languageMode="automatic-only"
      model="parakeet-tdt-v3"
      models={[]}
      onModelChange={vi.fn()}
    />);

    view.unmount();

    expect(dispatch).toHaveBeenCalledWith({
      action: "abandon-trial",
      trial_id: "00000000-0000-4000-8000-000000000001",
    });
  });
});
