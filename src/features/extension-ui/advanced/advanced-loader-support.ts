import { invoke } from "@tauri-apps/api/core";
import { IS_WINDOWS } from "@/lib/platform";
import type { AdvancedExtensionModule, AdvancedLoaderDependencies } from "./advanced-types";

export function parseModule(value: unknown): AdvancedExtensionModule {
  if (!value || typeof value !== "object"
    || typeof (value as AdvancedExtensionModule).activate !== "function") {
    throw new Error("extension_ui_activation_failed");
  }
  const module = value as AdvancedExtensionModule;
  if (module.deactivate !== undefined && typeof module.deactivate !== "function") {
    throw new Error("extension_ui_activation_failed");
  }
  return module;
}

export function artifactUrl(extensionId: string, hash: string, name: string): string {
  const base = IS_WINDOWS
    ? "http://beaver-extension.localhost"
    : "beaver-extension://localhost";
  return `${base}/${extensionId}/${hash}/${name}`;
}

export function mountStyle(document: Document, href: string, timeoutMs: number): Promise<HTMLLinkElement> {
  return new Promise((resolve, reject) => {
    const view = document.defaultView;
    if (!view) { reject(new Error("extension_ui_activation_failed")); return; }
    const link = document.createElement("link");
    const timeout = view.setTimeout(() => {
      link.remove();
      reject(new Error("extension_ui_activation_failed"));
    }, timeoutMs);
    link.rel = "stylesheet";
    link.href = href;
    link.onload = () => { view.clearTimeout(timeout); resolve(link); };
    link.onerror = () => {
      view.clearTimeout(timeout);
      link.remove();
      reject(new Error("extension_ui_activation_failed"));
    };
    document.head.append(link);
  });
}

export function limited<T>(
  promise: Promise<T>,
  timeoutMs: number,
  onLateValue?: (value: T) => void | Promise<void>,
): Promise<T> {
  return new Promise((resolve, reject) => {
    let expired = false;
    const timeout = window.setTimeout(() => {
      expired = true;
      reject(new Error("extension_ui_activation_failed"));
    }, timeoutMs);
    promise.then(
      (value) => {
        window.clearTimeout(timeout);
        if (expired) {
          void Promise.resolve(onLateValue?.(value)).catch(() => {});
        } else {
          resolve(value);
        }
      },
      (error) => {
        window.clearTimeout(timeout);
        if (!expired) {
          reject(error instanceof Error ? error : new Error("extension_ui_activation_failed"));
        }
      },
    );
  });
}

export function defaults(): AdvancedLoaderDependencies {
  return {
    document,
    importModule: (url) => import(/* @vite-ignore */ url),
    begin: (extensionId, attempts) => invoke("begin_extension_ui_load", { extensionId, attempts }),
    advance: (extensionId, token, stage) => invoke(
      "advance_extension_ui_load",
      { extensionId, token, stage },
    ),
    acknowledge: (extensionId, token) => invoke("acknowledge_extension_ui_load", { extensionId, token }),
    abort: (extensionId, token) => invoke("abort_extension_ui_load", { extensionId, token }),
  };
}
