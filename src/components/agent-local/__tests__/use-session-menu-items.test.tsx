import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useSessionMenuItems } from "../use-session-menu-items";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

let writeText: ReturnType<typeof vi.fn>;

beforeEach(() => {
  writeText = vi.fn(() => Promise.resolve());
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText },
  });
});

function menuFor(sessionId: string | null, pinned = false, onTogglePin = vi.fn()) {
  return renderHook(
    ({ id }: { id: string | null }) =>
      useSessionMenuItems({ sessionId: id, pinned, onRename: vi.fn(), onArchive: vi.fn(), onTogglePin }),
    { initialProps: { id: sessionId } },
  );
}

describe("commandes du menu d'une conversation", () => {
  it("n'expose rien tant qu'aucune conversation n'est visée", () => {
    const { result } = menuFor(null);

    expect(result.current).toEqual([]);
  });

  it("copie l'identifiant et annonce la réussite sans fermer le menu", async () => {
    const { result } = menuFor("session-42");
    expect(result.current[0].label).toBe("history.copyId");
    expect(result.current[0].keepOpen).toBe(true);

    await act(async () => { result.current[0].onClick(); await Promise.resolve(); });

    expect(writeText).toHaveBeenCalledWith("session-42");
    expect(result.current[0].label).toBe("history.idCopied");
    expect(result.current[0].keepOpen).toBe(true);
  });

  /* Le presse-papiers peut être refusé par le système. Annoncer « copié » dans
     ce cas ferait coller autre chose que l'identifiant attendu. */
  it("annonce l'échec quand le presse-papiers refuse", async () => {
    writeText.mockRejectedValueOnce(new Error("refusé"));
    const { result } = menuFor("session-42");

    await act(async () => { result.current[0].onClick(); await Promise.resolve(); });

    expect(result.current[0].label).toBe("history.copyIdFailed");
    expect(result.current[0].danger).toBe(true);
  });

  it("propose d'épingler une conversation qui ne l'est pas, avant l'archivage", () => {
    const onTogglePin = vi.fn();
    const { result } = menuFor("session-42", false, onTogglePin);
    const pin = result.current.find((item) => item.id === "pin");

    expect(pin?.label).toBe("history.pin");
    expect(result.current.map((item) => item.id)).toEqual(["copy-id", "rename", "pin", "archive"]);

    pin?.onClick();
    expect(onTogglePin).toHaveBeenCalledWith("session-42");
  });

  it("propose de désépingler une conversation épinglée", () => {
    const { result } = menuFor("session-42", true);

    expect(result.current.find((item) => item.id === "pin")?.label).toBe("history.unpin");
  });

  it("repart de la commande quand le menu s'ouvre sur une autre conversation", async () => {
    const { result, rerender } = menuFor("session-42");
    await act(async () => { result.current[0].onClick(); await Promise.resolve(); });
    expect(result.current[0].label).toBe("history.idCopied");

    rerender({ id: "session-7" });

    expect(result.current[0].label).toBe("history.copyId");
  });
});
