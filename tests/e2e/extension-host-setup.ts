import { invokeTauri } from "./tauri-invoke";
import { waitForHostReady } from "../../scripts/e2e/extension-host-ready";

export async function waitForExtensionHost(): Promise<void> {
  await waitForHostReady(
    () => invokeTauri("get_extension_host_status"),
    (condition, options) => browser.waitUntil(condition, options),
  );
}

let initialization: Promise<unknown> | undefined;

export async function initializeExtensionHost(): Promise<void> {
  // Grouped specs share one native process: Rust initialization is deliberately
  // single-use. Retain failures too, so a partial startup is never retried blindly.
  initialization ??= invokeTauri("e2e_initialize_extension_host");
  await initialization;
}
