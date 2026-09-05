import { invokeTauri } from "./tauri-invoke";

let initialization: Promise<unknown> | undefined;

export async function initializeExtensionHost(): Promise<void> {
  // Grouped specs share one native process: Rust initialization is deliberately
  // single-use. Retain failures too, so a partial startup is never retried blindly.
  initialization ??= invokeTauri("e2e_initialize_extension_host");
  await initialization;
}
