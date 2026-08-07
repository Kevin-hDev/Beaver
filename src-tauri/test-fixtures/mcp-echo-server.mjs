import { createInterface } from "node:readline";

const MAX_LINE_CHARS = 1_048_576;
const input = createInterface({ input: process.stdin, crlfDelay: Infinity });

input.on("line", (line) => {
  if (line.length > MAX_LINE_CHARS) process.exit(1);
  let request;
  try {
    request = JSON.parse(line);
  } catch {
    process.exit(1);
  }
  if (request.method === "notifications/initialized") return;

  let result = {};
  if (request.method === "tools/list") {
    result = {
      tools: [{
        name: "echo",
        description: "Echo a bounded test value",
        inputSchema: {
          type: "object",
          properties: { value: { type: "string" } },
          required: ["value"],
        },
      }],
    };
  }
  if (request.method === "tools/call") {
    const value = String(request.params?.arguments?.value ?? "").slice(0, 1024);
    result = { content: [{ type: "text", text: value }] };
  }
  process.stdout.write(`${JSON.stringify({ jsonrpc: "2.0", id: request.id, result })}\n`);
});
