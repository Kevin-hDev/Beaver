import { invoke } from "@tauri-apps/api/core";
import { IS_WINDOWS } from "@/lib/platform";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import type { ExtensionRecord } from "@/types/extensions";
import { sequenceExtensionUiLoad } from "../ui-load-sequencer";
import { createAdvancedContext } from "./advanced-context";
import { createAdvancedMountManager } from "./advanced-mounts";
import type {
  AdvancedCleanup,
  AdvancedExtensionModule,
  AdvancedLoaderDependencies,
  AdvancedLoaderInput,
} from "./advanced-types";

export async function loadAdvancedModules(
  input: AdvancedLoaderInput,
  dependencies: AdvancedLoaderDependencies = defaults(),
): Promise<AdvancedCleanup> {
  const loaded: AdvancedCleanup[] = [];
  try {
    for (const record of candidates(input.records, input.startup)) {
      if (!input.generationCurrent()) break;
      const cleanup = await sequenceExtensionUiLoad(() => loadOne(record, input, dependencies));
      if (!input.generationCurrent()) {
        await cleanup();
        break;
      }
      loaded.push(cleanup);
    }
  } catch (error) {
    await cleanupAll(loaded);
    throw error;
  }
  return () => cleanupAll(loaded);
}

async function loadOne(
  record: ExtensionRecord,
  input: AdvancedLoaderInput,
  dependencies: AdvancedLoaderDependencies,
): Promise<AdvancedCleanup> {
  const artifact = record.uiArtifact!;
  const extensionId = record.manifest.id;
  const attempts = retryAttempts(input.startup, extensionId);
  const token = await dependencies.begin(extensionId, attempts);
  const styles: HTMLLinkElement[] = [];
  const mounts = createAdvancedMountManager(dependencies.document);
  let activationCleanup: AdvancedCleanup | undefined;
  let module: AdvancedExtensionModule | undefined;
  try {
    await dependencies.advance(extensionId, "import");
    for (const output of artifact.outputs.filter(({ type }) => type === "css")) {
      styles.push(await mountStyle(
        dependencies.document,
        artifactUrl(extensionId, artifact.manifestSha256, output.name),
        UI_LIMITS.maxAdvancedActivationMs,
      ));
    }
    module = parseModule(await limited(
      dependencies.importModule(artifactUrl(extensionId, artifact.manifestSha256, artifact.entry)),
      UI_LIMITS.maxAdvancedActivationMs,
      async (lateValue) => {
        try { await parseModule(lateValue).deactivate?.(); } catch { /* résolution déjà expirée */ }
      },
    ));
    if (!input.generationCurrent()) throw new Error("extension_ui_activation_failed");
    await dependencies.advance(extensionId, "activate");
    const activationResult = await limited(
      Promise.resolve(module.activate(createAdvancedContext(extensionId, mounts))),
      UI_LIMITS.maxAdvancedActivationMs,
      async (lateValue) => {
        if (typeof lateValue === "function") {
          try { await lateValue(); } catch { /* cleanup tardif isolé */ }
        }
      },
    );
    if (activationResult !== undefined && typeof activationResult !== "function") {
      throw new Error("extension_ui_activation_failed");
    }
    activationCleanup = activationResult || undefined;
    if (!mounts.completed() || !input.generationCurrent()) {
      throw new Error("extension_ui_mount_failed");
    }
    await dependencies.advance(extensionId, "mount");
    await dependencies.acknowledge(extensionId, token);
  } catch (error) {
    await cleanupOne(module, activationCleanup, mounts.cleanup, styles);
    throw error;
  }
  return () => cleanupOne(module, activationCleanup, mounts.cleanup, styles);
}

function candidates(records: readonly ExtensionRecord[], startup: AdvancedLoaderInput["startup"]) {
  const retryId = startup.mode.kind === "retryInterruptedUi" ? startup.mode.extensionId : null;
  const allowAll = startup.mode.kind === "normal";
  return records.filter((record) => record.kind === "local"
    && record.enabled && record.trusted
    && record.manifest.apiLevel === "advanced"
    && record.manifest.ui?.mode === "advanced"
    && Boolean(record.uiArtifact)
    && (allowAll || record.manifest.id === retryId))
    .sort((left, right) => left.manifest.id.localeCompare(right.manifest.id));
}

function retryAttempts(startup: AdvancedLoaderInput["startup"], extensionId: string): number {
  return startup.mode.kind === "retryInterruptedUi" && startup.mode.extensionId === extensionId
    ? startup.mode.attempts
    : 1;
}

function parseModule(value: unknown): AdvancedExtensionModule {
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

function artifactUrl(extensionId: string, hash: string, name: string): string {
  const base = IS_WINDOWS
    ? "http://beaver-extension.localhost"
    : "beaver-extension://localhost";
  return `${base}/${extensionId}/${hash}/${name}`;
}

function mountStyle(document: Document, href: string, timeoutMs: number): Promise<HTMLLinkElement> {
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

async function cleanupOne(
  module: AdvancedExtensionModule | undefined,
  activation: AdvancedCleanup | undefined,
  mounts: AdvancedCleanup,
  styles: HTMLLinkElement[],
): Promise<void> {
  const tasks: AdvancedCleanup[] = [
    ...styles.map((style) => () => style.remove()),
    mounts,
  ];
  if (activation) tasks.push(activation);
  if (module?.deactivate) tasks.push(module.deactivate);
  await cleanupAll(tasks);
}

async function cleanupAll(cleanups: AdvancedCleanup[]): Promise<void> {
  for (const cleanup of cleanups.reverse()) {
    try { await cleanup(); } catch { /* le nettoyage suivant doit toujours s'exécuter */ }
  }
  cleanups.length = 0;
}

function limited<T>(
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

function defaults(): AdvancedLoaderDependencies {
  return {
    document,
    importModule: (url) => import(/* @vite-ignore */ url),
    begin: (extensionId, attempts) => invoke("begin_extension_ui_load", { extensionId, attempts }),
    advance: (extensionId, stage) => invoke("advance_extension_ui_load", { extensionId, stage }),
    acknowledge: (extensionId, token) => invoke("acknowledge_extension_ui_load", { extensionId, token }),
  };
}
