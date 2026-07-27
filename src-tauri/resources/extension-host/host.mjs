import { startProtocol } from "./protocol.mjs";
import { LIMITS } from "./contract.mjs";
import {
  callExtensionTool,
  emitExtensionEvent,
  syncExtensions,
} from "./loader.mjs";
import { JITI_VERSION } from "./versions.mjs";

for (const method of ["log", "info", "debug", "warn", "error"]) {
  console[method] = () => {};
}
// protocol.mjs already captured the real writer used exclusively for JSON-RPC.
process.stdout.write = () => true;
process.on("uncaughtException", () => process.exit(1));
process.on("unhandledRejection", () => process.exit(1));

startProtocol(async (method, params) => {
  switch (method) {
    case "host.hello":
      return {
        apiVersion: "1",
        jitiVersion: JITI_VERSION,
        nodeVersion: process.version,
      };
    case "host.sync": {
      const specifications = Array.isArray(params.extensions) ? params.extensions : [];
      if (specifications.length > LIMITS.maxExtensions) {
        throw new Error("too_many_extensions");
      }
      return syncExtensions(specifications);
    }
    case "tool.call":
      return callExtensionTool(String(params.name ?? ""), params.arguments ?? {});
    case "event.emit":
      return emitExtensionEvent(String(params.event ?? ""), params.payload ?? null);
    case "host.shutdown":
      setTimeout(() => process.exit(0), 0);
      return { stopping: true };
    default:
      throw new Error("unknown_method");
  }
});
