import { createJiti } from "jiti";
import { extname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const sdkPath = fileURLToPath(new URL("./sdk/index.mjs", import.meta.url));
const jiti = createJiti(import.meta.url, {
  alias: { "@beaver/sdk": sdkPath },
  fsCache: false,
  interopDefault: true,
  moduleCache: false,
  sourceMaps: false,
});
const nativeExtensions = new Set([".cjs", ".js", ".mjs"]);

export async function importExtensionModule(mainPath) {
  if (!nativeExtensions.has(extname(mainPath).toLowerCase())) {
    return jiti.import(mainPath, { default: true });
  }
  try {
    const namespace = await import(pathToFileURL(mainPath).href);
    return namespace.default ?? namespace;
  } catch (error) {
    if (!requiresJitiFallback(error)) throw error;
    return jiti.import(mainPath, { default: true });
  }
}

function requiresJitiFallback(error) {
  return error?.code === "ERR_UNKNOWN_FILE_EXTENSION"
    || (error?.code === "ERR_MODULE_NOT_FOUND"
      && String(error?.message).includes("@beaver/sdk"));
}
