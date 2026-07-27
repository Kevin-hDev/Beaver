import { spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import readline from "node:readline";

export function createHost(hostScript, options = {}) {
  const child = spawn(process.execPath, [hostScript], {
    shell: false,
    stdio: ["pipe", "pipe", "ignore"],
  });
  const pending = new Map();
  let closed = false;
  let resolveExit;
  const exited = new Promise((resolve) => { resolveExit = resolve; });
  const lines = readline.createInterface({ input: child.stdout });
  lines.on("line", (line) => {
    const message = JSON.parse(line);
    if (typeof message.method === "string") {
      const response = options.respondToCore?.(message)
        ?? (message.method === "app.info"
          ? { result: { apiVersion: "1" } }
          : {
              error: {
                code: -32_601,
                message: "core_method_unavailable",
              },
            });
      child.stdin.write(`${JSON.stringify({
        jsonrpc: "2.0",
        id: message.id,
        ...response,
      })}\n`);
      return;
    }
    const request = pending.get(message.id);
    if (!request) return;
    clearTimeout(request.timer);
    pending.delete(message.id);
    if (message.error) request.reject(new Error("host request failed"));
    else request.resolve(message.result);
  });
  child.once("exit", (code) => {
    closed = true;
    for (const request of pending.values()) {
      clearTimeout(request.timer);
      request.reject(new Error("host exited"));
    }
    pending.clear();
    resolveExit(code);
  });
  return {
    request(method, params) {
      if (closed) {
        return Promise.reject(new Error("host unavailable"));
      }
      if (pending.size >= 64) {
        return Promise.reject(new Error("too many pending host requests"));
      }
      const id = randomUUID();
      return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
          pending.delete(id);
          reject(new Error("host request timeout"));
        }, 5_000);
        pending.set(id, { resolve, reject, timer });
        child.stdin.write(`${JSON.stringify({
          jsonrpc: "2.0",
          id,
          method,
          params,
        })}\n`);
      });
    },
    stop() {
      child.kill();
    },
    exited,
  };
}
