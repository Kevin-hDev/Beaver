import type { AgentMessage } from "@/types/agent";

export function assistantWithSegmentTools(): AgentMessage {
  return {
    id: "assistant-1",
    role: "assistant",
    content: "",
    files: [],
    timestamp: new Date(0).toISOString(),
    segments: [
      {
        thinking: "first reflection",
        content: "",
        tools: [
          { name: "bash", summary: "npm test", result: "ok" },
          { name: "write_file", summary: "a.ts", result: "ok", content: "x" },
        ],
      },
      {
        content: "",
        tools: [{
          name: "edit_file",
          summary: "b.ts",
          result: "ok",
          old_text: "a",
          new_text: "b",
        }],
      },
      {
        thinking: "second reflection",
        content: "",
        tools: [{ name: "bash", summary: "npm run build", result: "ok" }],
      },
    ],
  };
}
