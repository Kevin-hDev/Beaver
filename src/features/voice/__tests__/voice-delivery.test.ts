/* @vitest-environment jsdom */
import { render, waitFor } from "@testing-library/react";
import { createElement } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  openComposerDraft,
  readComposerDraft,
  resetComposerDraftStoreForTests,
} from "@/hooks/composer-draft-store";
import type { VoiceDeliverySnapshot, VoiceSnapshot } from "@/types/voice.generated";
import { deliverVoiceResult } from "../voice-delivery";
import { VoiceRoot } from "../voice-root";
import { resetVoiceStoreForTests } from "../voice-store";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));
vi.mock("@/lib/platform", () => ({ IS_LINUX: false }));

const delivery: VoiceDeliverySnapshot = {
  id: "result-1",
  draftKey: "session:one",
  text: " dictée",
};
const snapshot = { revision: 2 } as VoiceSnapshot;

describe("voice delivery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetComposerDraftStoreForTests();
    resetVoiceStoreForTests();
    openComposerDraft("session:one");
    vi.mocked(listen).mockResolvedValue(vi.fn());
  });

  it("acquitte une insertion dans le brouillon courant", async () => {
    const acknowledge = vi.fn().mockResolvedValue(snapshot);
    await expect(deliverVoiceResult(delivery, acknowledge)).resolves.toBe(snapshot);
    expect(readComposerDraft("session:one").text).toBe(" dictée");
    expect(acknowledge).toHaveBeenCalledWith(delivery, "inserted");
  });

  it("ne duplique pas après un échec d'acquittement puis une relecture", async () => {
    await expect(deliverVoiceResult(delivery, vi.fn().mockRejectedValue(new Error("ipc"))))
      .rejects.toThrow("ipc");
    const acknowledge = vi.fn().mockResolvedValue(snapshot);
    await deliverVoiceResult(delivery, acknowledge);
    expect(readComposerDraft("session:one").text).toBe(" dictée");
    expect(acknowledge).toHaveBeenCalledWith(delivery, "already-inserted");
  });

  it("signale une destination fermée sans créer de texte", async () => {
    const acknowledge = vi.fn().mockResolvedValue(snapshot);
    await deliverVoiceResult({ ...delivery, draftKey: "session:closed" }, acknowledge);
    expect(readComposerDraft("session:closed").text).toBe("");
    expect(acknowledge).toHaveBeenCalledWith(
      { ...delivery, draftKey: "session:closed" },
      "closed",
    );
  });

  it("relit et acquitte sans duplication après un remount du pont", async () => {
    const pending = {
      revision: 1,
      phase: "delivering",
      operation: null,
      recovery: null,
      delivery,
      trialResult: null,
      error: null,
    } satisfies VoiceSnapshot;
    const idle = { ...pending, revision: 2, phase: "idle", delivery: null } satisfies VoiceSnapshot;
    let acknowledgements = 0;
    vi.mocked(invoke).mockImplementation((command: string) => {
      if (command === "voice_get_snapshot") return Promise.resolve(pending);
      acknowledgements += 1;
      return acknowledgements === 1 ? Promise.reject(new Error("ipc")) : Promise.resolve(idle);
    });

    const first = render(createElement(VoiceRoot));
    await waitFor(() => expect(readComposerDraft("session:one").text).toBe(" dictée"));
    await waitFor(() => expect(acknowledgements).toBe(1));
    first.unmount();
    render(createElement(VoiceRoot));

    await waitFor(() => expect(acknowledgements).toBe(2));
    expect(readComposerDraft("session:one").text).toBe(" dictée");
    expect(vi.mocked(listen).mock.invocationCallOrder[0])
      .toBeLessThan(vi.mocked(invoke).mock.invocationCallOrder[0]);
  });
});
