import { fatalProtocolExit, startProtocol } from "./protocol.mjs";
import { API_VERSION, LIMITS } from "./contract.mjs";
import {
  callExtensionTool,
  callExtensionUiAction,
  emitExtensionEvent,
  loadExtension,
  resetExtensions,
} from "./loader.mjs";
import { JITI_VERSION } from "./versions.mjs";

for (const method of ["log", "info", "debug", "warn", "error"]) {
  console[method] = () => {};
}
// protocol.mjs already captured the real writer used exclusively for JSON-RPC.
process.stdout.write = () => true;
process.on("uncaughtException", fatalProtocolExit);
process.on("unhandledRejection", fatalProtocolExit);

let loadedSinceReset = 0;

startProtocol(async (method, params) => {
  switch (method) {
    case "host.hello":
      return {
        apiVersion: API_VERSION,
        jitiVersion: JITI_VERSION,
        nodeVersion: process.version,
      };
    case "host.reset":
      loadedSinceReset = 0;
      return resetExtensions();
    case "host.load": {
      if (loadedSinceReset >= LIMITS.maxExtensions) {
        throw new Error("too_many_extensions");
      }
      loadedSinceReset += 1;
      return loadExtension(params.extension);
    }
    case "tool.call":
      return callExtensionTool(
        String(params.name ?? ""),
        params.arguments ?? {},
        params.context,
      );
    case "event.emit":
      return emitExtensionEvent(String(params.event ?? ""), params.payload ?? null);
    case "ui.action":
      return callExtensionUiAction(params);
    default:
      throw new Error("unknown_method");
  }
});
