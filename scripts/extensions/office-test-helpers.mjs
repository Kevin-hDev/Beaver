import { readFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createHost, resetAndLoad } from "./host-test-client.mjs";

export const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
export const hostDirectory = join(root, "src-tauri/target/extension-host");
export const hostScript = join(hostDirectory, "host.mjs");
export const fixtureDirectory = join(root, "scripts/extensions/fixtures");

export function createOfficeHost(options) {
  return createHost(hostScript, options);
}

export async function syncOfficePlugins(host) {
  const catalog = JSON.parse(
    await readFile(join(hostDirectory, "builtin-plugins/catalog.json"), "utf8"),
  );
  return resetAndLoad(host, catalog.plugins.map(({ manifest }) => ({
      id: manifest.id,
      mainPath: join(hostDirectory, manifest.main),
      manifest,
    })));
}

export function callOffice(host, workspace, name, arguments_) {
  return host.request("tool.call", {
    name,
    arguments: arguments_,
    context: { workingDirectory: workspace },
  });
}
