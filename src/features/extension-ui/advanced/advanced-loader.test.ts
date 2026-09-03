/* @vitest-environment jsdom */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ExtensionRecord, ExtensionUiStartupState } from "@/types/extensions";
import { resetExtensionUiLoadSequencerForTest } from "../ui-load-sequencer";
import { loadAdvancedModules } from "./advanced-loader";
import { advancedSlotAttributes } from "./advanced-mounts";
import type { AdvancedExtensionContext, AdvancedLoaderDependencies } from "./advanced-types";

const HASH = "a".repeat(64);
const NORMAL: ExtensionUiStartupState = {
  mode: { kind: "normal" },
  bootstrapResolved: true,
  thirdPartyLoadingAllowed: true,
  showRecoveryDialog: false,
  showSafeBanner: false,
  canRetry: false,
};

describe("advanced extension UI loader", () => {
  beforeEach(() => {
    document.head.replaceChildren();
    document.body.replaceChildren();
    resetExtensionUiLoadSequencerForTest();
  });

  afterEach(() => { vi.useRealTimers(); });

  it("loads extensions in stable order and acknowledges only after a mount", async () => {
    document.body.append(slot("app.toolbar.primary"));
    const events: string[] = [];
    const dependencies = harness(events, (url) => Promise.resolve({
      activate(context: AdvancedExtensionContext) {
        events.push(`activate:${context.extensionId}`);
        context.mount("app.toolbar.primary", (container) => {
          container.textContent = context.extensionId;
          events.push(`mount:${context.extensionId}`);
          return () => { events.push(`unmount:${context.extensionId}`); };
        });
        return () => { events.push(`cleanup:${context.extensionId}`); };
      },
      deactivate: () => { events.push(`deactivate:${extensionIdFrom(url)}`); },
    }));

    const cleanup = await loadAdvancedModules({
      records: [record("com.example.z"), record("com.example.a")],
      startup: NORMAL,
      generationCurrent: () => true,
    }, dependencies);

    expect(events.filter((event) => event.startsWith("begin:")))
      .toEqual(["begin:com.example.a", "begin:com.example.z"]);
    expect(events.indexOf("mount:com.example.a"))
      .toBeLessThan(events.indexOf("ack:com.example.a"));
    expect(document.querySelectorAll("[data-extension-ui-advanced-mount]")).toHaveLength(2);

    await cleanup();
    expect(document.querySelectorAll("[data-extension-ui-advanced-mount]")).toHaveLength(0);
    expect(events.slice(-3)).toEqual([
      "deactivate:com.example.a",
      "cleanup:com.example.a",
      "unmount:com.example.a",
    ]);
  });

  it("accepts an explicit no-mount activation and refuses an implicit one", async () => {
    const accepted = harness([], () => Promise.resolve({
      activate(context: AdvancedExtensionContext) { context.completeWithoutMounts(); },
    }));
    const cleanup = await loadAdvancedModules({
      records: [record("com.example.none")],
      startup: NORMAL,
      generationCurrent: () => true,
    }, accepted);
    expect(accepted.acknowledge).toHaveBeenCalledOnce();
    await cleanup();

    resetExtensionUiLoadSequencerForTest();
    const rejected = harness([], () => Promise.resolve({ activate() {} }));
    await expect(loadAdvancedModules({
      records: [record("com.example.missing")],
      startup: NORMAL,
      generationCurrent: () => true,
    }, rejected)).rejects.toThrow("extension_ui_mount_failed");
    expect(rejected.acknowledge).not.toHaveBeenCalled();
  });

  it("defers an unavailable session mount and remounts it when its anchor returns", async () => {
    const renders: HTMLElement[] = [];
    const cleanups: string[] = [];
    const dependencies = harness([], () => Promise.resolve({
      activate(context: AdvancedExtensionContext) {
        context.mount("agent.composer.leading", (container) => {
          renders.push(container);
          container.textContent = "mounted";
          return () => { cleanups.push("cleanup"); };
        });
      },
    }));

    const cleanup = await loadAdvancedModules({
      records: [record("com.example.deferred")],
      startup: NORMAL,
      generationCurrent: () => true,
    }, dependencies);
    expect(dependencies.acknowledge).toHaveBeenCalledOnce();
    expect(renders).toHaveLength(0);

    const first = document.createElement("span");
    first.setAttribute("data-extension-ui-slot", "agent.composer.leading");
    document.body.append(first);
    await vi.waitFor(() => expect(renders).toHaveLength(1));
    first.remove();
    await vi.waitFor(() => expect(cleanups).toHaveLength(1));

    const second = first.cloneNode() as HTMLElement;
    document.body.append(second);
    await vi.waitFor(() => expect(renders).toHaveLength(2));
    await cleanup();
    expect(cleanups).toHaveLength(2);
  });

  it("loads no third-party module in safe mode and only the retry target", async () => {
    const dependencies = harness([], () => Promise.resolve({
      activate(context: AdvancedExtensionContext) { context.completeWithoutMounts(); },
    }));
    await loadAdvancedModules({
      records: [record("com.example.a"), record("com.example.b")],
      startup: { ...NORMAL, mode: { kind: "safe", reason: "argument" } },
      generationCurrent: () => true,
    }, dependencies);
    expect(dependencies.begin).not.toHaveBeenCalled();

    await loadAdvancedModules({
      records: [record("com.example.a"), record("com.example.b")],
      startup: {
        ...NORMAL,
        mode: { kind: "retryInterruptedUi", extensionId: "com.example.b", attempts: 2 },
      },
      generationCurrent: () => true,
    }, dependencies);
    expect(dependencies.begin).toHaveBeenCalledWith("com.example.b", 2);
    expect(dependencies.begin).toHaveBeenCalledTimes(1);
  });

  it("closes the mount context after an activation timeout", async () => {
    vi.useFakeTimers();
    document.body.append(slot("app.toolbar.primary"));
    let captured: AdvancedExtensionContext | undefined;
    let resolveActivation!: () => void;
    const lateCleanup = vi.fn();
    const dependencies = harness([], () => Promise.resolve({
      activate(context: AdvancedExtensionContext) {
        captured = context;
        return new Promise<() => void>((resolve) => {
          resolveActivation = () => resolve(lateCleanup);
        });
      },
    }));
    const loading = loadAdvancedModules({
      records: [record("com.example.timeout")],
      startup: NORMAL,
      generationCurrent: () => true,
    }, dependencies);
    const refused = expect(loading).rejects.toThrow("extension_ui_activation_failed");
    await vi.advanceTimersByTimeAsync(15_000);

    await refused;
    expect(() => captured?.mount("app.toolbar.primary", () => {}))
      .toThrow("extension_ui_mount_failed");
    expect(dependencies.acknowledge).not.toHaveBeenCalled();
    resolveActivation();
    await vi.waitFor(() => expect(lateCleanup).toHaveBeenCalledOnce());
  });

  it("removes a stylesheet that exceeds the activation timeout", async () => {
    vi.useFakeTimers();
    const withStyle = record("com.example.style");
    withStyle.uiArtifact!.outputs.unshift({
      name: "style.css",
      type: "css",
      bytes: 1,
      sha256: HASH,
    });
    withStyle.uiArtifact!.totalBytes = 2;
    const dependencies = harness([], () => Promise.resolve({
      activate(context: AdvancedExtensionContext) { context.completeWithoutMounts(); },
    }));
    const loading = loadAdvancedModules({
      records: [withStyle],
      startup: NORMAL,
      generationCurrent: () => true,
    }, dependencies);
    const failure = loading.then(() => undefined, (error: unknown) => error);

    await vi.advanceTimersByTimeAsync(0);
    expect(document.head.querySelectorAll("link")).toHaveLength(1);
    await vi.advanceTimersByTimeAsync(15_000);
    expect(await failure).toEqual(new Error("extension_ui_activation_failed"));
    expect(document.head.querySelectorAll("link")).toHaveLength(0);
    expect(dependencies.importModule).not.toHaveBeenCalled();
  });
});

function record(id: string): ExtensionRecord {
  return {
    manifest: {
      id,
      name: id,
      version: "1.0.0",
      beaverApi: "1",
      runtime: "node",
      ui: { apiVersion: "1", mode: "advanced", entry: "entry.ts" },
      access: "full",
      apiLevel: "advanced",
      essential: false,
    },
    kind: "local",
    source: "/fixture",
    enabled: true,
    trusted: true,
    showInChat: false,
    status: "active",
    contributions: { tools: [], events: [] },
    uiArtifact: {
      version: 1,
      builderVersion: "0.28.1",
      nodeVersion: "v20.0.0",
      entry: "entry.js",
      totalBytes: 1,
      outputs: [{ name: "entry.js", type: "javascript", bytes: 1, sha256: HASH }],
      inputs: ["entry.ts"],
      manifestSha256: HASH,
    },
  };
}

function slot(placement: "app.toolbar.primary"): HTMLElement {
  const element = document.createElement("span");
  Object.entries(advancedSlotAttributes(placement))
    .forEach(([name, value]) => element.setAttribute(name, value));
  return element;
}

function harness(
  events: string[],
  importer: AdvancedLoaderDependencies["importModule"],
): AdvancedLoaderDependencies {
  return {
    document,
    importModule: vi.fn(importer),
    begin: vi.fn((id) => { events.push(`begin:${id}`); return Promise.resolve([1, 2, 3]); }),
    advance: vi.fn((id, _token, stage) => {
      events.push(`${stage}:${id}`);
      return Promise.resolve();
    }),
    acknowledge: vi.fn((id) => { events.push(`ack:${id}`); return Promise.resolve(); }),
  };
}

function extensionIdFrom(url: string): string {
  const parts = url.split("/");
  return parts[parts.length - 3] ?? "";
}
