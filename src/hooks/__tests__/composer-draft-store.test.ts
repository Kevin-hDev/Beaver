import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  acknowledgeVoiceDelivery,
  applyVoiceDelivery,
  closeComposerDraft,
  openComposerDraft,
  pinComposerDraft,
  readComposerDraft,
  resetComposerDraftStoreForTests,
  subscribeComposerDrafts,
  unpinComposerDraft,
  updateComposerDraft,
} from "../composer-draft-store";

function setText(key: string, text: string) {
  updateComposerDraft(key, (entry) => ({
    ...entry,
    text,
    skills: text.length === 0 ? [] : entry.skills,
  }));
}

describe("composer draft store", () => {
  beforeEach(resetComposerDraftStoreForTests);

  it("insère une livraison une seule fois dans le texte courant", () => {
    openComposerDraft("session:one");
    setText("session:one", "avant après");
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "session:one", text: "DICTÉ", microphoneDisconnected: false }))
      .toBe("inserted");
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "session:one", text: "DICTÉ", microphoneDisconnected: false }))
      .toBe("already-inserted");
    expect(readComposerDraft("session:one").text).toBe("avant aprèsDICTÉ");
  });

  it("ne crée pas de brouillon pour une destination fermée", () => {
    closeComposerDraft("session:closed");
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "session:closed", text: "DICTÉ", microphoneDisconnected: false }))
      .toBe("destination-closed");
    expect(readComposerDraft("session:closed").text).toBe("");
  });

  it("une écriture tardive ne rouvre pas une destination fermée", () => {
    openComposerDraft("session:closed");
    closeComposerDraft("session:closed");
    setText("session:closed", "retardataire");
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "session:closed", text: "DICTÉ", microphoneDisconnected: false }))
      .toBe("destination-closed");
  });

  it("protège une livraison appliquée jusqu'à son acquittement", () => {
    openComposerDraft("protected");
    expect(pinComposerDraft("protected", "operation-1")).toBe(true);
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "protected", text: "sauvé", microphoneDisconnected: false }))
      .toBe("inserted");
    unpinComposerDraft("protected", "operation-1");
    for (let index = 0; index < 64; index += 1) {
      openComposerDraft(`other:${index}`);
      setText(`other:${index}`, String(index));
    }
    expect(readComposerDraft("protected").text).toBe("sauvé");
    acknowledgeVoiceDelivery("protected", "result-1");
  });

  it("refuse une réservation si les 64 places sont déjà réservées", () => {
    for (let index = 0; index < 64; index += 1) {
      expect(pinComposerDraft(`pinned:${index}`, `operation:${index}`)).toBe(true);
    }
    expect(pinComposerDraft("overflow", "operation:overflow")).toBe(false);
  });

  it("conserve les abonnements existants quand la limite est atteinte", () => {
    const first = vi.fn();
    const cleanups = [subscribeComposerDrafts(first)];
    for (let index = 1; index < 64; index += 1) {
      cleanups.push(subscribeComposerDrafts(vi.fn()));
    }
    expect(() => subscribeComposerDrafts(vi.fn()))
      .toThrow("Active view subscription limit reached");
    openComposerDraft("listener-test");
    expect(first).toHaveBeenCalledOnce();
    cleanups.pop()?.();
    const replacement = subscribeComposerDrafts(vi.fn());
    replacement();
    cleanups.forEach((cleanup) => cleanup());
  });
});
