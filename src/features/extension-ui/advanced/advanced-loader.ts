import { LIMITS } from "@/types/extension-contract.generated";
import { cleanupAdvancedPlans } from "./advanced-cleanup";
import { artifactUrl, defaults, limited, mountStyle, parseModule } from "./advanced-loader-support";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import type { ExtensionRecord } from "@/types/extensions";
import { sequenceExtensionUiLoad } from "../ui-load-sequencer";
import { createAdvancedContext } from "./advanced-context";
import { createAdvancedMountManager } from "./advanced-mounts";
import type {
  AdvancedCleanup,
  AdvancedCleanupPlan,
  AdvancedExtensionModule,
  AdvancedLoaderDependencies,
  AdvancedLoaderInput,
} from "./advanced-types";

export async function loadAdvancedModules(
  input: AdvancedLoaderInput,
  dependencies: AdvancedLoaderDependencies = defaults(),
): Promise<AdvancedCleanup> {
  const loaded: AdvancedCleanupPlan[] = [];
  try {
    for (const record of candidates(input.records, input.startup)) {
      if (!input.generationCurrent()) break;
      const cleanup = await sequenceExtensionUiLoad(() => loadOne(record, input, dependencies, loaded));
      if (!input.generationCurrent()) {
        await cleanupAdvancedPlans([...loaded.splice(0), cleanup]);
        break;
      }
      loaded.push(cleanup);
    }
  } catch (error) {
    await cleanupAdvancedPlans(loaded.splice(0));
    throw error;
  }
  return () => cleanupAdvancedPlans(loaded.splice(0));
}

async function loadOne(
  record: ExtensionRecord,
  input: AdvancedLoaderInput,
  dependencies: AdvancedLoaderDependencies,
  loaded: AdvancedCleanupPlan[],
): Promise<AdvancedCleanupPlan> {
  const artifact = record.uiArtifact!;
  const extensionId = record.manifest.id;
  const attempts = retryAttempts(input.startup, extensionId);
  const token = await dependencies.begin(extensionId, attempts);
  const styles: HTMLLinkElement[] = [];
  const mounts = createAdvancedMountManager(dependencies.document);
  let activationCleanup: AdvancedCleanup | undefined;
  let deactivationCleanup: AdvancedCleanup | undefined;
  let module: AdvancedExtensionModule | undefined;
  let detached = false;
  const plan: AdvancedCleanupPlan = {
    detach: () => {
      if (detached) return [];
      detached = true;
      const callbacks = mounts.detach();
      for (const style of styles) style.remove();
      styles.length = 0;
      return [
        ...(deactivationCleanup ? [deactivationCleanup] : []),
        ...(activationCleanup ? [activationCleanup] : []),
        ...callbacks,
      ];
    },
  };
  try {
    await dependencies.advance(extensionId, token, "import");
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
    deactivationCleanup = module.deactivate;
    if (!input.generationCurrent()) throw new Error("extension_ui_activation_failed");
    await dependencies.advance(extensionId, token, "activate");
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
    await dependencies.advance(extensionId, token, "mount");
    await dependencies.acknowledge(extensionId, token);
  } catch (error) {
    await cleanupAdvancedPlans([...loaded.splice(0), plan]);
    // A caught failure has closed its context. Release only its authenticated attempt;
    // an interrupted process or synchronous stall cannot reach this path and keeps the journal.
    await dependencies.abort(extensionId, token).catch(() => {
      throw new Error("extension_ui_activation_failed");
    });
    throw error;
  }
  return plan;
}

function candidates(records: readonly ExtensionRecord[], startup: AdvancedLoaderInput["startup"]) {
  const retryId = startup.mode.kind === "retryInterruptedUi" ? startup.mode.extensionId : null;
  const allowAll = startup.mode.kind === "normal";
  return records.slice(0, LIMITS.maxExtensions).filter((record) => record.kind === "local"
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
