import extensionContract from "../../src-tauri/resources/extension-host/contract.json";
import type { ExtensionRecord } from "@/types/extensions";

export type ExtensionInstallSource = "git" | "npm";

export interface ExtensionInstallOutcome {
  record: ExtensionRecord | null;
  errorKey: string | null;
}

export const EXTENSION_INSTALL_LIMITS = Object.freeze({
  git: extensionContract.limits.maxGitLocatorChars,
  npm: extensionContract.limits.maxNpmSpecChars,
});
