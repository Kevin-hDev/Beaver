import { describe, it, expect, vi } from "vitest";
import { displaySessionName, idMatch } from "./utils";

describe("displaySessionName", () => {
  it("'Nouvelle session' → appelle t() et retourne sa valeur", () => {
    const t = vi.fn().mockReturnValue("Nouvelle session traduite");
    const result = displaySessionName("Nouvelle session", t);
    expect(t).toHaveBeenCalledWith("agentLocal.newSession");
    expect(result).toBe("Nouvelle session traduite");
  });

  it("'New session' → appelle t() et retourne sa valeur", () => {
    const t = vi.fn().mockReturnValue("New session translated");
    const result = displaySessionName("New session", t);
    expect(t).toHaveBeenCalledWith("agentLocal.newSession");
    expect(result).toBe("New session translated");
  });

  it("nom custom → retourne tel quel sans appeler t()", () => {
    const t = vi.fn();
    const result = displaySessionName("Mon projet secret", t);
    expect(t).not.toHaveBeenCalled();
    expect(result).toBe("Mon projet secret");
  });
});

describe("idMatch", () => {
  it("match exact → retourne true", () => {
    expect(idMatch("abc-123", "abc-123")).toBe(true);
  });

  it("null → retourne false", () => {
    expect(idMatch(null, "abc-123")).toBe(false);
  });

  it("undefined → retourne false", () => {
    expect(idMatch(undefined, "abc-123")).toBe(false);
  });
});
