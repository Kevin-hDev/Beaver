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

  it("charge le document versionné validé par Rust", async () => {
    invokeMock.mockResolvedValue(validDocument);

    await expect(loadSavedGroups()).resolves.toEqual(validDocument);
    expect(invokeMock).toHaveBeenCalledWith("load_terminal_tabs");
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
