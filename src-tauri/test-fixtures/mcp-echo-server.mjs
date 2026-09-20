import { createInterface } from "node:readline";

const MAX_LINE_CHARS = 1_048_576;
const initDelayMs = Math.min(
  Number.parseInt(process.env.BEAVER_MCP_INIT_DELAY_MS ?? "0", 10) || 0,
  1_000,
);
const input = createInterface({ input: process.stdin, crlfDelay: Infinity });

input.on("line", (line) => {
  if (line.length > MAX_LINE_CHARS) process.exit(1);
  let request;
  try {
    request = JSON.parse(line);
  } catch {
    process.exit(1);
  }
  if (request.method === "notifications/initialized" || request.id === undefined) return;

  let result;
  let error;
  switch (request.method) {
    case "initialize":
      if (request.params?.protocolVersion !== "2025-03-26") {
        error = { code: -32602, message: "Unsupported protocol version" };
        break;
      }
      result = {
        protocolVersion: "2025-03-26",
        capabilities: { tools: {} },
        serverInfo: { name: "beaver-test-echo", version: "1.0.0" },
      };
      break;
    case "tools/list":
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
      break;
    case "tools/call":
      if (request.params?.name !== "echo") {
        error = { code: -32602, message: "Unknown tool" };
        break;
      }
      result = { content: [{
        type: "text",
        text: String(request.params?.arguments?.value ?? "").slice(0, 1024),
      }] };
      break;
    default:
      error = { code: -32601, message: "Method not found" };
  }
  const respond = () => {
    process.stdout.write(`${JSON.stringify({ jsonrpc: "2.0", id: request.id, ...(error ? { error } : { result }) })}\n`);
  };
  if (request.method === "initialize" && initDelayMs > 0) {
    setTimeout(respond, initDelayMs);
  } else {
    respond();
  }
});
