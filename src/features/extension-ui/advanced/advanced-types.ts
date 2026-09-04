import type { ExtensionRecord, ExtensionUiStartupState } from "@/types/extensions";
import type { ExtensionUiPlacementKey } from "@/types/extension-ui-contract.generated";

export type AdvancedCleanup = () => void | Promise<void>;
export type AdvancedMount = (container: HTMLElement) => void | AdvancedCleanup;

export interface AdvancedExtensionContext {
  readonly apiVersion: string;
  readonly extensionId: string;
  mount(placement: ExtensionUiPlacementKey, mount: AdvancedMount): void;
  completeWithoutMounts(): void;
}

export interface AdvancedExtensionModule {
  activate(context: AdvancedExtensionContext): void | AdvancedCleanup | Promise<void | AdvancedCleanup>;
  deactivate?: AdvancedCleanup;
}

export interface AdvancedLoaderDependencies {
  document: Document;
  importModule: (url: string) => Promise<unknown>;
  begin: (extensionId: string, attempts: number) => Promise<number[]>;
  advance: (
    extensionId: string,
    token: number[],
    stage: "import" | "activate" | "mount",
  ) => Promise<void>;
  acknowledge: (extensionId: string, token: number[]) => Promise<void>;
}

export interface AdvancedLoaderInput {
  records: readonly ExtensionRecord[];
  startup: ExtensionUiStartupState;
  generationCurrent: () => boolean;
}
