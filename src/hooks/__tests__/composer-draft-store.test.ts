import { beforeEach, describe, expect, it } from "vitest";
import {
  acknowledgeVoiceDelivery,
  applyVoiceDelivery,
  closeComposerDraft,
  openComposerDraft,
  pinComposerDraft,
  readComposerDraft,
  resetComposerDraftStoreForTests,
  setComposerDraftText,
  unpinComposerDraft,
} from "../composer-draft-store";

describe("composer draft store", () => {
  beforeEach(resetComposerDraftStoreForTests);

  it("insère une livraison une seule fois dans le texte courant", () => {
    openComposerDraft("session:one");
    setComposerDraftText("session:one", "avant après");
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "session:one", text: "DICTÉ" }))
      .toBe("inserted");
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "session:one", text: "DICTÉ" }))
      .toBe("already-inserted");
    expect(readComposerDraft("session:one").text).toBe("avant aprèsDICTÉ");
  });

  it("ne crée pas de brouillon pour une destination fermée", () => {
    closeComposerDraft("session:closed");
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "session:closed", text: "DICTÉ" }))
      .toBe("destination-closed");
    expect(readComposerDraft("session:closed").text).toBe("");
  });

  it("une écriture tardive ne rouvre pas une destination fermée", () => {
    openComposerDraft("session:closed");
    closeComposerDraft("session:closed");
    setComposerDraftText("session:closed", "retardataire");
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "session:closed", text: "DICTÉ" }))
      .toBe("destination-closed");
  });

  it("protège une livraison appliquée jusqu'à son acquittement", () => {
    openComposerDraft("protected");
    expect(pinComposerDraft("protected", "operation-1")).toBe(true);
    expect(applyVoiceDelivery({ id: "result-1", draftKey: "protected", text: "sauvé" }))
      .toBe("inserted");
    unpinComposerDraft("protected", "operation-1");
    for (let index = 0; index < 64; index += 1) {
      openComposerDraft(`other:${index}`);
      setComposerDraftText(`other:${index}`, String(index));
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
});
