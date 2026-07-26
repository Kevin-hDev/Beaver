import { beforeEach, describe, expect, it, vi } from "vitest";
import { resolveAgentStreamMessages } from "../agent-stream-message-resolver";
import type { AgentMessage, FileAttachment } from "@/types/agent";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn(),
  readFile: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));
vi.mock("@tauri-apps/plugin-fs", () => ({ readFile: mocks.readFile }));

function message(file: FileAttachment): AgentMessage {
  return {
    id: "m1",
    role: "user",
    content: "Pièce jointe",
    files: [file],
    timestamp: "2026-07-13T10:00:00Z",
  };
}

describe("attachment access restoration", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("garde la miniature sans relire une ancienne pièce jointe", async () => {
    const resolved = await resolveAgentStreamMessages([message({
      name: "legacy.png",
      path: "/tmp/legacy.png",
      mime_type: "image/png",
      size: 4,
      thumbnail: "data:image/png;base64,bGVnYWN5",
    })]);

    expect(mocks.invoke).not.toHaveBeenCalled();
    expect(mocks.readFile).not.toHaveBeenCalled();
    expect(resolved[0].images).toEqual(["bGVnYWN5"]);
  });

  it("refuse de lire lorsque Rust rejette la preuve", async () => {
    mocks.invoke.mockRejectedValue(new Error("denied"));
    const resolved = await resolveAgentStreamMessages([message({
      name: "forged.txt",
      path: "/tmp/forged.txt",
      mime_type: "text/plain",
      size: 4,
      access_grant: "v1.forged",
    })]);

    expect(mocks.invoke).toHaveBeenCalledWith("restore_attachment_access", {
      path: "/tmp/forged.txt",
      accessGrant: "v1.forged",
    });
    expect(mocks.readFile).not.toHaveBeenCalled();
    expect(resolved[0].content).toBe("Pièce jointe");
  });

  it("relit le fichier seulement après validation de la preuve", async () => {
    mocks.invoke.mockResolvedValue(undefined);
    mocks.readFile.mockResolvedValue(new TextEncoder().encode("contenu sûr"));
    const resolved = await resolveAgentStreamMessages([message({
      name: "notes.txt",
      path: "/tmp/notes.txt",
      mime_type: "text/plain",
      size: 12,
      access_grant: "v1.valid",
    })]);

    expect(mocks.invoke).toHaveBeenCalledBefore(mocks.readFile);
    expect(resolved[0].content).toContain("contenu sûr");
  });
});
