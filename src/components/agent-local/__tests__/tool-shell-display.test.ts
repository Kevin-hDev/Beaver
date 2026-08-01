import { describe, expect, it } from "vitest";
import {
  isLegacyShellStopError,
  isShellStopAction,
  shellCommandPreview,
} from "../tool-shell-display";

describe("tool shell display", () => {
  it("reconnaît stop=true et Ctrl+C comme des arrêts volontaires", () => {
    expect(isShellStopAction({
      name: "bash_write",
      summary: "npm start",
      args: { stop: true },
    })).toBe(true);
    expect(isShellStopAction({
      name: "bash_write",
      summary: "npm start",
      args: { chars: "\u0003" },
    })).toBe(true);
    expect(isShellStopAction({
      name: "bash_write",
      summary: "npm start",
      args: { chars: "hello" },
    })).toBe(false);
  });

  it("utilise la commande exacte fournie par le backend", () => {
    expect(shellCommandPreview({
      name: "bash_write",
      summary: "npm run dev -- --host 127.0.0.1",
      args: { session_id: "session-1", stop: true },
    }, [])).toBe("npm run dev -- --host 127.0.0.1");
  });

  it("reclasse seulement l'ancien faux échec d'un arrêt volontaire", () => {
    const stop = {
      name: "bash_write",
      summary: "session-1",
      args: { session_id: "session-1", stop: true },
      result: "Commande annulee.",
    };
    expect(isLegacyShellStopError(stop, true)).toBe(true);
    expect(isLegacyShellStopError({ ...stop, result: "Session shell introuvable." }, true)).toBe(false);
    expect(isLegacyShellStopError(stop, false)).toBe(false);
  });

  it("retrouve la commande des anciennes sessions à partir de leur identifiant", () => {
    const sessionId = "6a719eeb-1665-49cd-a5e2-23427e80543b";
    expect(shellCommandPreview({
      name: "bash_write",
      summary: sessionId,
      args: { session_id: sessionId, chars: "\u0003" },
    }, [{
      name: "bash",
      summary: "npm start",
      result: `[Processus actif: session_id=${sessionId}, pid=91661, 1001 ms]`,
    }])).toBe("npm start");
  });
});
