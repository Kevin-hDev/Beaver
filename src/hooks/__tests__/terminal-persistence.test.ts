import { beforeEach, describe, expect, it, vi } from "vitest";
import { loadSavedGroups, saveGroups } from "../terminal-persistence";
import type { TerminalTabsDocument } from "../terminal-persistence";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

const validDocument: TerminalTabsDocument = {
  version: 1,
  groups: { project: [{ label: "build" }] },
};

describe("terminal persistence IPC", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("charge et valide le document versionné renvoyé par Rust", async () => {
    invokeMock.mockResolvedValue(validDocument);

    await expect(loadSavedGroups()).resolves.toEqual(validDocument);
    expect(invokeMock).toHaveBeenCalledWith("load_terminal_tabs");
  });

  it.each([
    undefined,
    null,
    [],
    {},
    { version: 1 },
    { groups: {} },
    { version: 2, groups: {} },
    { version: 1, groups: [] },
    { version: 1, groups: { project: [{ label: "" }] } },
    { version: 1, groups: { project: [{ label: "bad\nlabel" }] } },
    { version: 1, groups: { project: [{ label: "é".repeat(257) }] } },
    { version: 1, groups: { project: Array.from({ length: 17 }, () => ({ label: "tab" })) } },
    {
      version: 1,
      groups: Object.fromEntries(Array.from({ length: 129 }, (_, index) => [`g-${index}`, []])),
    },
    {
      version: 1,
      groups: Object.fromEntries(Array.from({ length: 17 }, (_, group) => [
        `g-${group}`,
        Array.from({ length: 16 }, (_, tab) => ({ label: `${group}-${tab}` })),
      ])),
    },
  ])("rejette une réponse IPC invalide au lieu de créer un document vide", async (value) => {
    invokeMock.mockResolvedValue(value);

    await expect(loadSavedGroups()).rejects.toThrow("terminal-tabs-invalid");
  });

  it("accepte un ancien libellé composé d'espaces lorsque Rust l'a validé", async () => {
    const document = { version: 1, groups: { project: [{ label: " " }] } };
    invokeMock.mockResolvedValue(document);

    await expect(loadSavedGroups()).resolves.toEqual(document);
  });

  it("transmet uniquement le document durable à Rust", async () => {
    invokeMock.mockResolvedValue(undefined);

    await saveGroups(validDocument);

    expect(invokeMock).toHaveBeenCalledWith("save_terminal_tabs", {
      document: validDocument,
    });
  });
});
