import { startProtocol } from "./protocol.mjs";
import {
  callExtensionTool,
  emitExtensionEvent,
  syncExtensions,
} from "./loader.mjs";

const MAX_EXTENSIONS = 128;

for (const method of ["log", "info", "debug", "warn", "error"]) {
  console[method] = () => {};
}
process.on("uncaughtException", () => process.exit(1));
process.on("unhandledRejection", () => process.exit(1));

startProtocol(async (method, params) => {
  switch (method) {
    case "host.hello":
      return {
        apiVersion: "1",
        jitiVersion: "2.7.0",
        nodeVersion: process.version,
      };
    case "host.sync": {
      const specifications = Array.isArray(params.extensions) ? params.extensions : [];
      if (specifications.length > MAX_EXTENSIONS) throw new Error("too_many_extensions");
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
