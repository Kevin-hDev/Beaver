import { beforeEach, describe, expect, it, vi } from "vitest";
import { resolveAgentStreamMessages } from "../agent-stream-message-resolver";
import type { AgentMessage } from "@/types/agent";

vi.mock("@tauri-apps/plugin-fs", () => ({
  readFile: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

function message(
  role: AgentMessage["role"],
  content: string,
  extra: Partial<AgentMessage> = {},
): AgentMessage {
  return {
    id: crypto.randomUUID(),
    role,
    content,
    files: [],
    timestamp: "2026-07-20T01:00:00Z",
    ...extra,
  };
}

describe("liaison des outils restaurés", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("relie un résultat sans identifiant à son appel après compression", async () => {
    const resolved = await resolveAgentStreamMessages([
      message("assistant", "Je consulte le fichier.", {
        tool_calls: [{
          function: {
            name: "read_file",
            arguments: { path: "notes.md" },
          },
        }],
      }),
      message("tool", "Contenu du fichier", { tool_name: "read_file" }),
      message("assistant", "Voici la réponse."),
      message("user", "Continue"),
    ]);

    const toolCall = resolved[0].tool_calls?.[0] as { id?: string };
    expect(toolCall.id).toMatch(/^[0-9a-f-]{36}$/);
    expect(resolved[1].tool_call_id).toBe(toolCall.id);
  });
});
